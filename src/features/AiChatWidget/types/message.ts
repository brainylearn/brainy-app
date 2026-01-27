export default interface Message {
	from: "bot" | "human";
	content: string;
}
