import { invoke } from "@tauri-apps/api/core";
import FsrsProfile from "../types/backend/entity/fsrsProfile";

export function getAllFsrsProfiles(): Promise<FsrsProfile[]> {
	return invoke("get_all_fsrs_profiles");
}

export function getFileFsrsProfile(id: string): Promise<FsrsProfile> {
	return invoke("get_file_fsrs_profile", { id });
}

export function getFolderFsrsProfile(id: string): Promise<FsrsProfile> {
	return invoke("get_folder_fsrs_profile", { id });
}

export function getParentProfileForFile(id: string): Promise<FsrsProfile> {
	return invoke("get_parent_profile_for_file", { id });
}

export function getParentProfileForFolder(id: string): Promise<FsrsProfile> {
	return invoke("get_parent_profile_for_folder", { id });
}

export function createProfile(profile: {
	name: string;
	requestRetention: number;
	maximumInterval: number;
	weights: number[];
}): Promise<FsrsProfile> {
	return invoke("create_profile", profile);
}

export function updateProfile(profile: FsrsProfile): Promise<void> {
	return invoke("update_profile", { ...profile });
}
