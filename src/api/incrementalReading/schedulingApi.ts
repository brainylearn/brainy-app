import { invoke } from "@tauri-apps/api/core";

export function scheduleIncrementalReadingLater(
	cellId: string,
	nextReadingDate: Date,
): Promise<void> {
	return invoke("schedule_incremental_reading_later", {
		cellId,
		nextReadingDate: nextReadingDate.toISOString(),
	});
}
