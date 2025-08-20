import FileRepetitionCounts from "../model/fileRepetitionCounts";

export default interface FileWithRepetitionCounts {
	id: string;
	path: string;
	isFolder: boolean;
	repetitionCounts?: FileRepetitionCounts;
}
