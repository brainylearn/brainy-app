export type IncrementalReadingPriority = "high" | "normal" | "low";

export default interface IncrementalReading {
	content: string | null;
	title: string | null;
	priority: IncrementalReadingPriority;
}
