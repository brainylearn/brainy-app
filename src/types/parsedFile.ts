import FileRepetitionCounts from "./backend/model/fileRepetitionCounts";

export default interface ParsedFile {
	id: string;
	name: string;
	repetitionCounts: FileRepetitionCounts;
}
