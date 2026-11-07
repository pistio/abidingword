import { act, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { ApiProvider } from "@/lib/ApiContext";
import { PdfStructurePanel } from "@/features/library/PdfStructurePanel";
import type { PdfBuildProgress, PdfDiagnostic, PdfStructureView } from "@/types/library";
import { createMemoryApi } from "../support/memoryApi";

const empty: PdfStructureView = {sourceId:"source",fileHash:"hash",status:"missing",complete:true,tagged:false,unalignedTags:0,uncertainPages:0,headings:[],corrections:[],canUndo:false};
const running: PdfBuildProgress = {status:"running",stage:"pdf",done:125,total:400,elapsedMs:12000,timings:[{stage:"file",milliseconds:3}],headingCount:null,indexedLinks:null,errorCode:null,canCancel:true};
function panel(api: ReturnType<typeof createMemoryApi>) {
  const changed=vi.fn(async()=>{}), close=vi.fn();
  const ui=render(<ApiProvider api={api}><PdfStructurePanel sourceId="source" segments={[]} onChanged={changed} onGo={()=>{}} onPage={()=>{}} onClose={close} observed={()=>({mode:"original",physicalPage:24})}/></ApiProvider>);
  return {...ui,changed,close};
}
function setup() {
  const api=createMemoryApi(); api.libraryPdfStructure=async()=>empty;
  return api;
}
describe("observable PDF analysis",()=>{
  it("shows page progress and elapsed time, permits close, and retains a failure on reopening",async()=>{
    const api=setup();let progress:PdfBuildProgress|null=running;
    api.libraryPdfStructureProgress=async()=>progress;
    const first=panel(api);
    await screen.findByText("Read PDF pages · 12 s");
    expect(screen.getByRole("progressbar")).toHaveAttribute("value","125");
    expect(screen.getByRole("button",{name:"Build structure"})).toBeDisabled();
    const user=userEvent.setup();await user.click(screen.getAllByRole("button",{name:"Close"}).at(-1)!);
    expect(first.close).toHaveBeenCalled();first.unmount();
    progress={...running,status:"failed",errorCode:"library.structure.timeout",canCancel:false};
    panel(api);await screen.findByText(/reached its time limit/);
    expect(screen.getByRole("button",{name:"Build structure"})).toBeEnabled();
    expect(screen.queryByText(/headings saved/)).not.toBeInTheDocument();
  });
  it("requests cancellation and reports rollback without pretending the build succeeded",async()=>{
    const api=setup();let progress:PdfBuildProgress|null=running;
    api.libraryPdfStructureProgress=async()=>progress;
    api.libraryPdfStructureCancel=vi.fn(async()=>{progress={...running,status:"cancelled",canCancel:false};return true;});
    panel(api);const user=userEvent.setup();await screen.findByRole("progressbar");
    await user.click(screen.getByRole("button",{name:"Cancel"}));
    await screen.findByText(/Analysis cancelled/);
    expect(api.libraryPdfStructureCancel).toHaveBeenCalledWith("source");
    expect(screen.getByRole("button",{name:"Build structure"})).toBeEnabled();
  });
  it("reports saved zero-heading metadata as unusable Contents, with manual correction still offered",async()=>{
    const api=setup();api.libraryPdfStructure=async()=>({...empty,status:"ready"});
    api.libraryPdfStructureProgress=async()=>({...running,status:"empty",headingCount:0,indexedLinks:0,canCancel:false});
    panel(api);await screen.findByText(/No usable headings were recognised/);
    expect(screen.queryByText(/headings saved/)).not.toBeInTheDocument();
    expect(screen.getByText("Mark a visible text line as a section")).toBeVisible();
  });
  it("displays the exact saved report, labels general status, and clears stale results when the query changes",async()=>{
    const api=setup();const base=await api.libraryPdfStructureDiagnostic("source",null,null);
    const saved:PdfDiagnostic={...base,source:{...base.source,fileSha256:"SAVED-FINGERPRINT"},query:null};
    api.chooseDiagnosticFile=vi.fn(async()=>"/chosen/report.json");
    api.libraryPdfStructureDiagnosticSave=vi.fn(async()=>({path:"/chosen/report.json",report:saved}));
    panel(api);const user=userEvent.setup();await screen.findByText(/No saved heading analysis/);
    await user.click(screen.getByText("Diagnostic report (metadata only)"));
    await user.click(screen.getByRole("button",{name:"Save report…"}));
    await screen.findByText(/SAVED-FINGERPRINT/);
    expect(screen.getByText(/General status report/)).toBeVisible();
    expect(api.libraryPdfStructureDiagnosticSave).toHaveBeenCalledWith("source",null,{mode:"original",physicalPage:24},"/chosen/report.json");
    await user.type(screen.getByRole("textbox",{name:"Passage to check (optional)"}),"Apg 6");
    expect(screen.queryByText(/SAVED-FINGERPRINT/)).not.toBeInTheDocument();
    expect(screen.getByRole("button",{name:"Copy report"})).toBeDisabled();
    await user.click(screen.getByRole("button",{name:"Save report…"}));
    await waitFor(()=>expect(api.libraryPdfStructureDiagnosticSave).toHaveBeenLastCalledWith("source","Apg 6",{mode:"original",physicalPage:24},"/chosen/report.json"));
  });
  it("disables duplicate builds immediately while waiting for the first progress poll",async()=>{
    const api=setup();let finish!:(v:PdfStructureView)=>void;
    api.libraryPdfStructureBuild=vi.fn(()=>new Promise<PdfStructureView>(resolve=>{finish=resolve;}));
    const view=panel(api);const user=userEvent.setup();await screen.findByText(/No saved heading analysis/);
    await user.click(screen.getByRole("button",{name:"Build structure"}));
    expect(screen.getByRole("button",{name:"Build structure"})).toBeDisabled();
    await act(async()=>finish({...empty,status:"ready"}));
    await waitFor(()=>expect(view.changed).toHaveBeenCalledOnce());
  });
});
