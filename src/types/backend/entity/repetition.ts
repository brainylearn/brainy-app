export type RepetitionState = "new" | "learning" | "relearning" | "review";

// TODO: missing fields like created and modified date makes an error, the same is for other entities, find a solution
export default interface Repetition {
	id: string;
	fileId: string;
	cellId: string;
	due: string;
	stability: number;
	difficulty: number;
	elapsedDays: number;
	scheduledDays: number;
	reps: number;
	lapses: number;
	state: RepetitionState;
	lastReview: string | null;
	additionalContent: string | null;
}
