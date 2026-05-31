import {
	$getSelection,
	$isRangeSelection,
	COMMAND_PRIORITY_EDITOR,
	createCommand,
	LexicalCommand,
	RangeSelection,
} from "lexical";
import { useLexicalComposerContext } from "@lexical/react/LexicalComposerContext";
import { useEffect } from "react";
import {
	$createHighlightNode,
	$isHighlightNode,
	$isSelectionInsideHighlight,
	HIGHLIGHT_TAG_NAME,
	HighlightNode,
} from "./highlightNode";
import {
	$removeSelectionFromNode,
	$wrapSelectionInNode,
} from "../../../../../../components/RichTextEditor/Plugins/utils/selectionWrapUtils";

export const TOGGLE_HIGHLIGHT_NODE: LexicalCommand<void> = createCommand();

export function HighlightPlugin() {
	const [editor] = useLexicalComposerContext();

	useEffect(() => {
		if (!editor.hasNodes([HighlightNode])) {
			throw new Error("HighlightNode not registered on editor");
		}

		const unregisterToggleHighlight = editor.registerCommand(
			TOGGLE_HIGHLIGHT_NODE,
			() => {
				editor.update(() => {
					const selection = $getSelection();
					if (
						!$isRangeSelection(selection) ||
						selection.isCollapsed()
					) {
						return;
					}

					if ($isSelectionInsideHighlight(selection)) {
						$removeSelectionFromHighlight(selection);
					} else {
						$wrapSelectionInHighlight(selection);
					}
				});
				return true;
			},
			COMMAND_PRIORITY_EDITOR,
		);

		return () => {
			unregisterToggleHighlight();
		};
	}, [editor]);

	return null;
}

function $wrapSelectionInHighlight(selection: RangeSelection): HighlightNode {
	return $wrapSelectionInNode(
		selection,
		$isHighlightNode,
		existing => $createHighlightNode(existing?.cellIds ?? []),
		HIGHLIGHT_TAG_NAME,
	);
}

function $removeSelectionFromHighlight(selection: RangeSelection) {
	$removeSelectionFromNode(selection, $isHighlightNode, existing =>
		$createHighlightNode(existing.cellIds),
	);
}
