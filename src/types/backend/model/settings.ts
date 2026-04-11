export type Theme = "FollowSystem" | "Light" | "Dark";

// TODO: should be called settings dto
export default interface Settings {
	databaseDirectory: string;

	theme: Theme;
	zoomPercentage: number;
	autoSync: boolean;

	enableAi: boolean;
	ollamaModelName: string | null;
	ollamaEmbeddingsModelName: string | null;
}
