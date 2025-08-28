import { invoke } from "@tauri-apps/api/core";
import FileRepetitionCounts from "../types/backend/model/fileRepetitionCounts";
import Repetition from "../types/backend/entity/repetition";

export function getStudyRepetitionCounts(
	fileId: string,
): Promise<FileRepetitionCounts> {
	return invoke("get_study_repetition_counts", {
		fileId,
	});
}

export function getFileRepetitions(fileId: string): Promise<Repetition[]> {
	return invoke("get_file_repetitions", {
		fileId,
	});
}

export function getRepetitionsForFiles(
	fileIds: string[],
): Promise<Repetition[]> {
	return invoke("get_repetitions_for_files", { fileIds });
}

export function resetRepetitionsForCell(cellId: number) {
	return invoke("reset_repetitions_for_cell", { cellId });
}
