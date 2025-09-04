import { invoke } from "@tauri-apps/api/core";
import Cell from "../types/backend/entity/cell";
import UpdateCellRequest from "../types/backend/dto/updateCellRequest";

export function getFileCellsOrderedByIndex(fileId: string): Promise<Cell[]> {
	return invoke("get_file_cells_ordered_by_index", {
		fileId,
	});
}

export function updateCellsContents(requests: UpdateCellRequest[]) {
	return invoke("update_cells_contents", { requests });
}

export function createCell(cell: Cell): Promise<string> {
	return invoke("create_cell", { ...cell });
}

export function deleteCell(cellId: string) {
	return invoke("delete_cell", { cellId });
}

export function moveCell(cellId: string, newIndex: number) {
	return invoke("move_cell", {
		cellId,
		newIndex,
	});
}

export function getCellsForFiles(fileIds: string[]): Promise<Cell[]> {
	return invoke("get_cells_for_files", { fileIds });
}
