import { DialogFilter } from "@tauri-apps/plugin-dialog";

export const JSON_FILE_FILTER: DialogFilter = {
	name: "*.json",
	extensions: ["json"],
};

export const FILE_ITEM_SOURCE_DATA = "FILE_ITEM_SOURCE_DATA";
export const FILE_ITEM_TARGET_DATA = "FILE_ITEM_TARGET_DATA";
