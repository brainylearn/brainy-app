import { invoke } from "@tauri-apps/api/core";

export function exportItem(itemId: string, exportPath: string) {
	return invoke("export", {
		itemId,
		exportPath,
	});
}

export function importFile(importItemPath: string, importIntoFolderId: string) {
	return invoke("import", {
		importItemPath,
		importIntoFolderId,
	});
}
