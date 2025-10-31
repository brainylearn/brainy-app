import {
	mdiFormatBold,
	mdiFormatItalic,
	mdiFormatListBulleted,
	mdiFormatListNumbered,
	mdiFormatSubscript,
	mdiFormatSuperscript,
	mdiFormatUnderline,
} from "@mdi/js";
import { FORMAT_TEXT_COMMAND } from "lexical";
import {
	$isListNode,
	INSERT_ORDERED_LIST_COMMAND,
	INSERT_UNORDERED_LIST_COMMAND,
	REMOVE_LIST_COMMAND,
} from "@lexical/list";
import { IFloatingMenuButton } from "./FloatingMenuButton";

export const defaultButtons: IFloatingMenuButton[] = [
	{
		name: "bold",
		title: "Bold (Ctrl + B)",
		icon: mdiFormatBold,
		onClick: editor => editor.dispatchCommand(FORMAT_TEXT_COMMAND, "bold"),
		isActive: selection => selection.hasFormat("bold"),
	},
	{
		name: "italic",
		title: "Italic (Ctrl + I)",
		icon: mdiFormatItalic,
		onClick: editor =>
			editor.dispatchCommand(FORMAT_TEXT_COMMAND, "italic"),
		isActive: selection => selection.hasFormat("italic"),
	},
	{
		name: "underline",
		title: "Underline (Ctrl + U)",
		icon: mdiFormatUnderline,
		onClick: editor =>
			editor.dispatchCommand(FORMAT_TEXT_COMMAND, "underline"),
		isActive: selection => selection.hasFormat("underline"),
	},
	{
		name: "orderedList",
		title: "Ordered list (Ctrl + Shift + 7)",
		icon: mdiFormatListNumbered,
		onClick: (editor, isActive) =>
			editor.dispatchCommand(
				isActive ? REMOVE_LIST_COMMAND : INSERT_ORDERED_LIST_COMMAND,
				undefined,
			),
		isActive: selection => {
			for (const node of selection.getNodes()) {
				let current = node.getParent();
				while (current !== null) {
					if (
						$isListNode(current) &&
						current.getListType() === "number"
					) {
						return true;
					}
					current = current.getParent();
				}
			}
			return false;
		},
	},
	{
		name: "bulletList",
		title: "Bullet list (Ctrl + Shift + 8)",
		icon: mdiFormatListBulleted,
		onClick: (editor, isActive) =>
			editor.dispatchCommand(
				isActive ? REMOVE_LIST_COMMAND : INSERT_UNORDERED_LIST_COMMAND,
				undefined,
			),
		isActive: selection => {
			for (const node of selection.getNodes()) {
				let current = node.getParent();
				while (current !== null) {
					if (
						$isListNode(current) &&
						current.getListType() === "bullet"
					) {
						return true;
					}
					current = current.getParent();
				}
			}
			return false;
		},
	},
	{
		name: "subscript",
		title: "Subscript (Ctrl + ,)",
		icon: mdiFormatSubscript,
		onClick: editor =>
			editor.dispatchCommand(FORMAT_TEXT_COMMAND, "subscript"),
		isActive: selection => selection.hasFormat("subscript"),
	},
	{
		name: "superscript",
		title: "Superscript (Ctrl + .)",
		icon: mdiFormatSuperscript,
		onClick: editor =>
			editor.dispatchCommand(FORMAT_TEXT_COMMAND, "superscript"),
		isActive: selection => selection.hasFormat("superscript"),
	},
];
