const ONE_DAY_IN_MILLISECONDS = 86400000;

export default function formatDueDate(isoDate: string): string {
	const date = new Date(isoDate);
	const now = new Date();
	const startOfToday = new Date(
		now.getFullYear(),
		now.getMonth(),
		now.getDate(),
	);
	const startOfDate = new Date(
		date.getFullYear(),
		date.getMonth(),
		date.getDate(),
	);
	const diffDays = Math.round(
		(startOfDate.getTime() - startOfToday.getTime()) /
			ONE_DAY_IN_MILLISECONDS,
	);

	if (diffDays < 0) return "today";
	if (diffDays === 0) return "today";
	if (diffDays === 1) return "tomorrow";
	return date.toLocaleDateString(undefined, {
		month: "short",
		day: "numeric",
	});
}
