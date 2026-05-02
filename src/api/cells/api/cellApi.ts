import { invoke } from "@tauri-apps/api/core";
import Cell from "../entities/cell";
import UpdateCellRequest from "../dto/updateCellRequest";
import { CellWithFsrsProfileId } from "../dto/cellWithFsrsProfileId";

export function getFileCellsOrderedByIndex(fileId: string): Promise<Cell[]> {
	return invoke("get_file_cells_ordered_by_index", {
		fileId,
	});
}

export function updateCellsContents(requests: UpdateCellRequest[]) {
	return invoke("update_cells_contents", { requests });
}

// TODO: should get its own dto request
export function createCell(cell: Cell): Promise<string> {
	return invoke("create_cell", { ...cell });
}

export function deleteCell(id: string) {
	return invoke("delete_cell", { id });
}

export function moveCell(id: string, newIndex: number) {
	return invoke("move_cell", {
		id,
		newIndex,
	});
}

export function getCellsForFilesWithFsrsProfileIds(
	fileIds: string[],
): Promise<CellWithFsrsProfileId[]> {
	return invoke("get_cells_for_files_with_fsrs_profile_ids", { fileIds });
}
