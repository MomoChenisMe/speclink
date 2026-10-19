import {beforeEach,expect,it,vi} from "vitest";
import {PROJECT_COLORS,readProjectColors,writeProjectColors} from "../projectColors";
import {projectTileClass} from "../components/projectTile";
beforeEach(()=>localStorage.clear());
it("依 locator 保留顏色，關閉重開不遺失，壞值忽略",()=>{
 writeProjectColors({"local:/a":"blue","remote:c/p/r":"purple"});
 expect(readProjectColors()).toEqual({"local:/a":"blue","remote:c/p/r":"purple"});
 localStorage.setItem("speclink.projectColors",'{"local:/a":"not-a-color","local:/b":"green"}');
 expect(readProjectColors()).toEqual({"local:/b":"green"});
 localStorage.setItem("speclink.projectColors","bad");expect(readProjectColors()).toEqual({});
});
it("儲存被拒絕時回報錯誤，不冒稱成功",()=>{
 const storage={setItem:vi.fn(()=>{throw new Error("denied");})} as unknown as Storage;
 expect(()=>writeProjectColors({"local:/a":"blue"},storage)).toThrow("denied");
});
it.each(PROJECT_COLORS)("%s 未選取淺底黑字，選取維持深底白字",(color)=>{
 expect(projectTileClass(color,false).split(" ")).toContain(`bg-project-${color}-muted`);
 expect(projectTileClass(color,false).split(" ")).toContain(`text-project-${color}-muted-foreground`);
 expect(projectTileClass(color,true).split(" ")).toContain(`bg-project-${color}`);
 expect(projectTileClass(color,true).split(" ")).toContain("text-project-foreground");
});
it("紫色和粉紅色保持不同色系；預設深灰與淺灰",()=>{
 expect(projectTileClass("purple",false)).not.toBe(projectTileClass("pink",false));
 expect(projectTileClass(undefined,true)).toContain("bg-project-default");
 expect(projectTileClass(undefined,false)).toContain("bg-project-default-muted");
});
