import { ReviewTreeFolder } from "../../api/fileSystem/dto/reviewTreeFolder";
import UiFile from "./uiFile";

export default interface UiFolder extends ReviewTreeFolder {
	subfolders: UiFolder[];
	files: UiFile[];
	isVisible: boolean;
}
