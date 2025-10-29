import { useEffect } from "react";
import {
	$insertList,
	$removeList,
	INSERT_ORDERED_LIST_COMMAND,
	INSERT_UNORDERED_LIST_COMMAND,
	REMOVE_LIST_COMMAND,
} from "@lexical/list";
import { useLexicalComposerContext } from "@lexical/react/LexicalComposerContext";
import { COMMAND_PRIORITY_LOW } from "lexical";

export default function ListCommandsPlugin() {
	const [editor] = useLexicalComposerContext();

	useEffect(() => {
		const unregisterListeners: (() => void)[] = [];
		unregisterListeners.push(
			editor.registerCommand(
				INSERT_UNORDERED_LIST_COMMAND,
				() => {
					$insertList("bullet");
					return true;
				},
				COMMAND_PRIORITY_LOW,
			),
		);

		unregisterListeners.push(
			editor.registerCommand(
				INSERT_ORDERED_LIST_COMMAND,
				() => {
					$insertList("number");
					return true;
				},
				COMMAND_PRIORITY_LOW,
			),
		);

		unregisterListeners.push(
			editor.registerCommand(
				REMOVE_LIST_COMMAND,
				() => {
					$removeList();
					return true;
				},
				COMMAND_PRIORITY_LOW,
			),
		);

		return () => {
			for (const unregisterListener of unregisterListeners) {
				unregisterListener();
			}
		};
	}, [editor]);

	return <></>;
}
