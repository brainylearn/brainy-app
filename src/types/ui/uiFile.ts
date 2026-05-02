import { ReviewTreeFile } from "../../api/fileSystem/dto/reviewTreeFolder";

export default interface UiFile extends ReviewTreeFile {
	isVisible: boolean;
}
