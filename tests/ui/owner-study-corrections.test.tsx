import { act, cleanup, screen, waitFor, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import i18n from "@/i18n";
import { createMemoryApi } from "../support/memoryApi";
import { renderAppInProject as renderApp } from "../support/render";
import { openPartOptions } from "../support/studyActions";
import { sectionTitle } from "@/features/studies/builtinText";
import pozetek from "../../resources/templates/pozetek-writing-study.json";
const pane=()=>screen.getByRole("main",{name:"Study"});
async function fixture(old=false){
 await act(async()=>{await i18n.changeLanguage("en");});
 const api=createMemoryApi();const project=await api.projectCreate({workspaceId:api.state.workspace.id,title:"Owner correction fixtures"});
 const template=api.state.templates.find(t=>t.key==="pozetek-writing-study")!;
 const {study}=await api.studyCreate({projectId:project.id,title:"Review study",templateId:template.id});
 if(old){
  for(const [key,former] of [["context","theme"],["explanations","understanding"],["core","consequences"]]){
   const section=api.state.sections.find(s=>s.studyId===study.id && s.key===key)!;section.key=former!;section.title=`Saved ${former}`;
   await api.studyBlockUpdate({id:api.state.blocks.find(b=>b.sectionId===section.id)!.id,content:`Preserve ${former} 한글`});
  }
  const extra=await api.studySectionCreate({studyId:study.id,title:"My separate Time notes"});api.state.sections.find(s=>s.id===extra.id)!.key="time";
 }
 const open=async()=>{const result=await renderApp(api);const listButton=await screen.findByRole("button",{name:/^Review study/}).catch(()=>null);if(listButton)await result.user.click(listButton);await screen.findByRole("textbox",{name:"Study title"});return result;};
 return {api,study,open};
}
describe("owner Study corrections",()=>{
 it.each(["en","de","ko","fr","es","pt"])("keeps exactly the original seven keys and POZETEK identity in %s",async language=>{
  await act(async()=>{await i18n.changeLanguage(language);});
  const parts=pozetek.sections.filter(s=>!s.optional);
  expect(parts.map(s=>s.key)).toEqual(["persons","place","time","events","context","explanations","core"]);
  expect(i18n.t("studies.builtin.templates.pozetek-writing-study.name").includes("POZETEK")).toBe(language === "de");
  for(const part of parts){expect(sectionTitle(part,pozetek.key)).not.toMatch(/^studies\./);expect(i18n.t(`studies.builtin.templates.pozetek-writing-study.sections.${part.key}.guidance`, {defaultValue:""}).length).toBeGreaterThan(120);}
 });
 it("starts with meaningful dismissible hints, keeps dismissal after reopening, and preserves full autosaved writing in a resizable field",async()=>{
  const {api,open}=await fixture();const {user}=await open();
  const part=within(pane()).getByRole("heading",{name:"Persons"}).closest("section")!;
  expect(within(part).getByText(/book’s author and original audience/)).toBeVisible();
  expect(within(part).queryByText(/Write in your own words/)).toBeNull();
  const answer=within(part).getByRole("textbox");expect(answer).toHaveAttribute("rows","5");expect(answer).toHaveClass("resize-y");
  await user.click(within(part).getByRole("button",{name:"Hide hints"}));expect(part.querySelector("[data-part-hints]")).toBeNull();
  const text="Long original writing 한글 ".repeat(20);await user.type(answer,text);await user.tab();
  await waitFor(()=>expect(api.state.blocks.some(b=>b.content===text)).toBe(true));
  cleanup();await open();const again=within(pane()).getByRole("heading",{name:"Persons"}).closest("section")!;
  expect(again.querySelector("[data-part-hints]")).toBeNull();expect(within(again).getByRole("textbox")).toHaveValue(text);
  await openPartOptions(user,"Persons");await user.click(within(again).getByRole("button",{name:"Hints"}));expect(again.querySelector("[data-part-hints]")).not.toBeNull();
 });
 it("always asks in a named modal for an empty part, traps focus, keeps on Escape and supports empty-part undo",async()=>{
  const {api,study,open}=await fixture();const {user}=await open();const remove=vi.spyOn(api,"studySectionDelete");
  await openPartOptions(user,"Persons");const opener=within(pane()).getByRole("button",{name:"Delete part Persons"});await user.click(opener);
  let modal=screen.getByRole("alertdialog",{name:"Delete part Persons?"});expect(modal).toHaveAttribute("aria-modal","true");expect(remove).not.toHaveBeenCalled();
  const keep=within(modal).getByRole("button",{name:"Keep it"});expect(keep).toHaveFocus();
  await user.keyboard("{Shift>}{Tab}{/Shift}");expect(within(modal).getByRole("button",{name:"Delete part"})).toHaveFocus();
  await user.keyboard("{Tab}");expect(keep).toHaveFocus();
  await user.keyboard("{Escape}");expect(opener).toHaveFocus();expect((await api.studySectionList(study.id))).toHaveLength(7);
  await user.click(opener);modal=screen.getByRole("alertdialog");await user.click(within(modal).getByRole("button",{name:"Delete part"}));
  await waitFor(()=>expect(api.state.sections.filter(s=>s.studyId===study.id)).toHaveLength(6));
  await user.click(await screen.findByRole("button",{name:"Undo"}));await waitFor(()=>expect(api.state.sections.filter(s=>s.studyId===study.id)).toHaveLength(7));
 });
 it("previews old and duplicate parts, keeps all writing, repairs on consent and undoes",async()=>{
  const {api,study,open}=await fixture(true);const before=(await api.studySectionList(study.id)).map(s=>({id:s.id,title:s.title,key:s.key,blocks:s.blocks}));const {user}=await open();
  const apply=vi.spyOn(api,"studyPozetekRepair");await user.click(within(pane()).getByRole("button",{name:"Review POZETEK parts"}));
  let modal=await screen.findByRole("dialog",{name:"Review POZETEK parts"});expect(within(modal).getByRole("status")).toHaveTextContent("time (2)");
  expect(apply.mock.calls.some(call=>call[1]==="apply")).toBe(false);
  await user.click(within(modal).getByRole("button",{name:"Cancel"}));expect((await api.studySectionList(study.id)).map(s=>s.id)).toEqual(before.map(s=>s.id));
  await user.click(within(pane()).getByRole("button",{name:"Review POZETEK parts"}));modal=await screen.findByRole("dialog");await user.click(within(modal).getByRole("button",{name:"Add missing parts and order"}));
  await screen.findByRole("button",{name:"Undo POZETEK repair"});const after=await api.studySectionList(study.id);
  expect(after.slice(0,7).map(s=>s.key)).toEqual(["persons","place","time","events","context","explanations","core"]);
  for(const original of before)expect(after.find(s=>s.id===original.id)).toMatchObject(original);
  await openPartOptions(user,"Text context");
  const newPart=within(pane()).getByRole("heading",{name:"Text context"}).closest("section")!;
  await user.click(within(newPart).getByRole("button",{name:"Note — Text context"}));
  await user.click(screen.getByRole("menuitem",{name:"Observation"}));
  await user.type(within(newPart).getByLabelText("New observation"),"Unsaved draft must survive");
  expect(screen.getByRole("button",{name:"Undo POZETEK repair"})).toBeDisabled();
  expect(within(newPart).getByLabelText("New observation")).toHaveValue("Unsaved draft must survive");
  await user.click(within(newPart).getByRole("button",{name:"Cancel"}));
  await user.click(within(newPart).getByRole("button",{name:"Rename part Text context"}));
  const name=within(newPart).getByRole("textbox",{name:"Heading of the part"});
  await user.clear(name);await user.type(name,"Unsubmitted name must survive");
  await user.click(screen.getByRole("button",{name:"Undo POZETEK repair"}));
  expect(name).toHaveValue("Unsubmitted name must survive");
  expect(apply.mock.calls.some(call=>call[1]==="undo")).toBe(false);
  await user.click(within(newPart).getByRole("button",{name:"Cancel"}));

  await user.click(screen.getByRole("button",{name:"Undo POZETEK repair"}));await waitFor(()=>expect(api.state.sections.filter(s=>s.studyId===study.id)).toHaveLength(before.length));
  expect((await api.studySectionList(study.id)).map(s=>s.id)).toEqual(before.map(s=>s.id));
 });
});
