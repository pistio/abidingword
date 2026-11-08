import { readerPageName } from "../support/readerActions";
import { screen, within, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { PdfAnchor, PdfResolvedHeading, PdfStructureView } from "@/types/library";
import { createMemoryApi } from "../support/memoryApi";
import { createFakePdfEngine } from "../support/fakePdfEngine";
import { renderApp } from "../support/render";
import { readerAction } from "../support/readerActions";

const anchor:PdfAnchor={page:2,pageWidth:600,pageHeight:800,x0:70,x1:280,baseline:250,size:20,firstChar:100,column:0,byteStart:0,byteEnd:12,originalText:"Apg 6,1–7"};
const text="Apg 6,1–7\nAn invented observation — 한글 remains exact.";
const heading:PdfResolvedHeading={id:"p2-c100",title:"Apg 6,1–7",level:2,parents:["Apostelgeschichte"],basis:"outline",anchor,segmentPosition:1,start:0,end:9,disabled:false,mapping:null,mappingScope:null,canonical:"ACT.6.1-ACT.6.7",scope:"passage",endPosition:3,path:"Apostelgeschichte › Apg 6,1–7"};
async function open(ready=true, shared=false, delayed=false) {
  const actualText=shared?`Acts\n${text}`:text;
  const actualHeading=shared?{...heading,start:5,end:14}:heading;
  const earlier={...heading,id:"p2-c0",title:"Acts",level:1,start:0,end:4,anchor:{...anchor,baseline:100,size:24,firstChar:0,originalText:"Acts"}};
  const headings=shared?[earlier,actualHeading]:[heading];
  const api=createMemoryApi();api.installFixtureBible("FXA","alpha");
  const source=api.installFixtureSource({title:"Invented structured commentary",format:"pdf",language:"Deutsch",segments:[
    {text:"Apostelgeschichte",pageNumber:1},
    {text:actualText,pageNumber:2,presentation:{role:"paragraph",basis:"layout",uncertain:false,printedLabel:"101",anchors:[anchor],headings,sectionPath:heading.path,runs:[]}},
    {text:"Kapitel 7. A later section cites Apg 6,1 incidentally.",pageNumber:3},
  ]});
  let view:PdfStructureView={sourceId:source.id,fileHash:"hash",status:ready?"ready":"missing",complete:true,tagged:false,unalignedTags:0,uncertainPages:0,headings:ready?headings:[],corrections:[],canUndo:false};
  let release=()=>{};const pending=new Promise<void>(resolve=>{release=resolve;});
  api.libraryPdfStructure=async()=>{if(delayed)await pending;return view;};
  api.librarySourceContents=async()=>view.headings.map(h=>({id:h.id,title:h.title,target:{kind:"segment",position:h.segmentPosition},children:[]}));
  api.libraryPdfStructureBuild=vi.fn(async()=>{view={...view,status:"ready",headings:[heading]};return view;});
  api.libraryPdfStructureCorrect=vi.fn(async(_id,_hash,correction)=>{view={...view,headings:[{...heading,title:correction.title,canonical:correction.mapping}],corrections:[correction],canUndo:true};return view;});
  api.libraryPdfStructureUndo=vi.fn(async()=>{view={...view,headings:[heading],corrections:[],canUndo:false};return view;});
  if(shared) {
    api.state.settings.scripturePassage="ACT.6";
    const segments=await api.sourceGetSegments({sourceId:source.id});
    api.libraryPassageSources=async()=>[{sourceId:source.id,sourceTitle:source.title,author:null,moreLocations:0,locations:[{segmentId:segments.segments[1]!.id,locator:"p. 2",origin:"section",matchKind:"contained",matchedText:heading.title,sectionHeading:heading.title,structureNodeId:heading.id,structureBasis:"outline",structureScope:"passage",reference:{canonical:"ACT.6.1-ACT.6.7",display:"Acts 6:1–7",book:"ACT",startChapter:6,startVerse:1,endChapter:6,endVerse:7}}]}];
  }
  const engine=createFakePdfEngine({pages:3,outline:[{id:"raw",title:"Uncorrected PDF outline",target:{kind:"page",page:3},children:[]}]});
  const {user}=await renderApp(api,{pdfEngine:engine,width:900});
  await user.click(screen.getByRole("button",{name:"Show Sources pane"}));
  if(!shared) await user.click(await screen.findByRole("button",{name:"Open library"}));
  await user.click(await screen.findByRole("button",{name:/^Invented structured commentary/}));
  if(shared) await screen.findByRole("heading",{name:"Apg 6,1–7"});
  else await screen.findByRole("heading",{name:"Invented structured commentary"});
  return {api,engine,user,source,release};
}
describe("PDF structure in the complete reader",()=>{
  it("keeps a visible centred section when preceding-page context is also on screen during a direct Original switch",async()=>{
    const {user}=await open(true,true);
    const body=screen.getByTestId("reader-body");
    vi.spyOn(body,"getBoundingClientRect").mockReturnValue(new DOMRect(0,0,300,600));
    const first=body.querySelector<HTMLElement>('[data-seg-position="0"]')!;
    vi.spyOn(first,"getBoundingClientRect").mockReturnValue(new DOMRect(0,10,300,80));
    const target=body.querySelector<HTMLElement>('[data-seg-position="1"]')!;
    vi.spyOn(target,"getBoundingClientRect").mockReturnValue(new DOMRect(0,150,300,300));
    const selected=target.querySelector<HTMLElement>('[data-pdf-heading="p2-c100"]')!;
    vi.spyOn(selected,"getBoundingClientRect").mockReturnValue(new DOMRect(0,200,300,40));
    await user.click(screen.getByRole("button",{name:"Original"}));
    await waitFor(()=>expect(screen.getByTestId("pdf-page-indicator")).toHaveAccessibleName(readerPageName("2")));
  });
  it("targets the requested heading when several sections share a paragraph, including Contents and history",async()=>{
    const scrolled:HTMLElement[]=[];
    const original=Element.prototype.scrollIntoView;
    Element.prototype.scrollIntoView=function(this:HTMLElement){scrolled.push(this);};
    try {
      const {user}=await open(true,true);
      await waitFor(()=>expect(scrolled.some(el=>el.dataset.pdfHeading===heading.id)).toBe(true));
      scrolled.length=0;
      await user.click(await screen.findByRole("button",{name:"Contents"}));
      await user.click(await screen.findByRole("button",{name:"Acts"}));
      await waitFor(()=>expect(scrolled.some(el=>el.dataset.pdfHeading==="p2-c0")).toBe(true));
      scrolled.length=0;
      await user.click(screen.getByRole("button",{name:"Contents"}));
      await user.click(await screen.findByRole("button",{name:"Apg 6,1–7"}));
      await waitFor(()=>expect(scrolled.some(el=>el.dataset.pdfHeading===heading.id)).toBe(true));
      expect(document.querySelector('[data-seg-position="1"]')?.textContent).toBe(`Acts\n${text}`);
    } finally {Element.prototype.scrollIntoView=original;}
  });
  it("styles the anchored heading without changing stored quotation text and uses corrected segment Contents",async()=>{
    const {user,engine}=await open();
    expect(await screen.findByRole("heading",{name:"Apg 6,1–7"})).toBeVisible();
    expect(document.querySelector('[data-seg-position="1"]')?.textContent).toBe(text);
    expect(screen.getByText(/101.*PDF page 2/)).toBeVisible();
    await user.click(await screen.findByRole("button",{name:"Contents"}));
    expect(await screen.findByRole("button",{name:"Apg 6,1–7"})).toBeVisible();
    expect(screen.queryByRole("button",{name:"Uncorrected PDF outline"})).toBeNull();
    expect(engine.outlineReads).toHaveLength(0);
  });
  it("sends an anchored, hash-bound correction and refreshes Contents; undo remains discoverable",async()=>{
    const {user,api,source}=await open();
    await readerAction(user, "Chapters and headings");
    const dialog=await screen.findByRole("dialog",{name:"Chapters and headings"});
    expect(within(dialog).getByText(/automatically recognised headings and chapters/)).toBeVisible();
    await user.click(await within(dialog).findByRole("button",{name:"Correct section"}));
    const title=within(dialog).getByRole("textbox",{name:"Section title"});await user.clear(title);await user.type(title,"A corrected section title");
    await user.click(within(dialog).getByRole("button",{name:"Save"}));
    await waitFor(()=>expect(api.libraryPdfStructureCorrect).toHaveBeenCalledWith(source.id,"hash",expect.objectContaining({id:heading.id,position:1,anchorId:"p2-c100",start:0,end:9,title:"A corrected section title",mapping:"ACT.6.1-ACT.6.7"})));
    await waitFor(()=>expect(within(dialog).getByRole("button",{name:"Undo correction"})).toBeEnabled());
    await user.click(within(dialog).getByRole("button",{name:"Undo correction"}));
    await waitFor(()=>expect(api.libraryPdfStructureUndo).toHaveBeenCalledWith(source.id,"hash",false));
    await user.click(within(dialog).getByRole("button",{name:"Close"}));
    expect(document.querySelector('[data-seg-position="1"]')?.textContent).toBe(text);
  });
  it("offers explicit structure building for an older import without doing it on opening",async()=>{
    const {user,api}=await open(false);
    expect(api.libraryPdfStructureBuild).not.toHaveBeenCalled();
    await readerAction(user, "Chapters and headings");
    const dialog=await screen.findByRole("dialog",{name:"Chapters and headings"});
    await user.click(await within(dialog).findByRole("button",{name:"Build structure"}));
    await waitFor(()=>expect(api.libraryPdfStructureBuild).toHaveBeenCalledOnce());
    expect(await within(dialog).findByRole("button",{name:"Correct section"})).toBeVisible();
  });
  it("keeps the corrected Contents after Original supplies an obsolete outline",async()=>{
    const {user}=await open();
    await user.click(await screen.findByRole("button",{name:"Original"}));
    await user.click(await screen.findByRole("button",{name:"Contents"}));
    expect(await screen.findByRole("button",{name:"Apg 6,1–7"})).toBeVisible();
    expect(screen.queryByRole("button",{name:"Uncorrected PDF outline"})).toBeNull();
    await user.click(screen.getByRole("button",{name:"Apg 6,1–7"}));
    await user.click(screen.getByRole("button",{name:"Reading"}));
    expect(await screen.findByRole("heading",{name:"Apg 6,1–7"})).toBeVisible();
  });
  it("preserves supplied page bookmarks separately from corrected section targets",async()=>{
    const {user}=await open();
    await readerAction(user, "Chapters and headings");
    const dialog=await screen.findByRole("dialog",{name:"Chapters and headings"});
    await user.click(within(dialog).getByText("Original PDF bookmarks"));
    expect(await within(dialog).findByRole("button",{name:/Uncorrected PDF outline/})).toBeVisible();
    expect(within(dialog).getByText(/without the app’s section corrections/)).toBeVisible();
    await user.click(within(dialog).getByRole("button",{name:"Close"}));
    await user.click(screen.getByRole("button",{name:"Contents"}));
    expect(await screen.findByRole("button",{name:"Apg 6,1–7"})).toBeVisible();
    expect(screen.queryByRole("button",{name:"Uncorrected PDF outline"})).toBeNull();
  });

  it("replaces an early raw outline when the corrected metadata arrives later",async()=>{
    const {user,release}=await open(true,false,true);
    await user.click(screen.getByRole("button",{name:"Original"}));
    await user.click(await screen.findByRole("button",{name:"Contents"}));
    expect(await screen.findByRole("button",{name:"Uncorrected PDF outline"})).toBeVisible();
    release();
    expect(await screen.findByRole("button",{name:"Apg 6,1–7"})).toBeVisible();
    await waitFor(()=>expect(screen.queryByRole("button",{name:"Uncorrected PDF outline"})).toBeNull());
  });

  it("continues a few paragraphs beyond an indexed hit when expanding the pane reader",async()=>{
    const scrolled:HTMLElement[]=[];const original=Element.prototype.scrollIntoView;
    Element.prototype.scrollIntoView=function(this:HTMLElement){scrolled.push(this);};
    try {
      const {user}=await open(true,true);
      const body=screen.getByTestId("reader-body");
      vi.spyOn(body,"getBoundingClientRect").mockReturnValue(new DOMRect(0,0,300,600));
      for(const [position,y] of [[0,-400],[1,-200],[2,0]]) {
        const element=body.querySelector<HTMLElement>(`[data-seg-position="${position}"]`)!;
        vi.spyOn(element,"getBoundingClientRect").mockReturnValue(new DOMRect(0,y,300,100));
      }
      scrolled.length=0;
      await readerAction(user, "Open full reader");
      await screen.findByRole("heading",{name:"Invented structured commentary"});
      await waitFor(()=>expect(scrolled.some(el=>el.dataset.segPosition==="2")).toBe(true));
      const last=scrolled.at(-1)!;
      expect(last.closest<HTMLElement>("[data-seg-position]")?.dataset.segPosition).toBe("2");
    } finally {Element.prototype.scrollIntoView=original;}
  });

});
