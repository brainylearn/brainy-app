import { invoke } from "@tauri-apps/api/core";
import HomeStatistics from "../models/homeStatistics";
import { Rating } from "../entities/rating";
import RepetitionUpdate from "../valueObjects/repetitionUpdate";

export function registerReview(
	repetitionUpdate: RepetitionUpdate,
	rating: Rating,
	studyTime: number,
) {
	return invoke("register_review", { repetitionUpdate, rating, studyTime });
}

export function getHomeStatistics(): Promise<HomeStatistics> {
	return invoke("get_home_statistics");
}
