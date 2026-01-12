import { invoke } from "@tauri-apps/api/core";
import FsrsProfile from "../types/backend/entity/fsrsProfile";
import { ItemFsrsProfile } from "../types/backend/value_objects/itemFsrsProfile";

export function getAllFsrsProfiles(): Promise<FsrsProfile[]> {
	return invoke("get_all_fsrs_profiles");
}

export function getFileFsrsProfile(id: string): Promise<ItemFsrsProfile> {
	return invoke("get_file_fsrs_profile", { id });
}

export function getFolderFsrsProfile(id: string): Promise<ItemFsrsProfile> {
	return invoke("get_folder_fsrs_profile", { id });
}
