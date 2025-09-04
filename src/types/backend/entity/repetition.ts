export type RepetitionState = "New" | "Learning" | "Relearning" | "Review";

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
	lastReview: string;
	additionalContent?: string;
}
