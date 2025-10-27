import { computePosition } from "@floating-ui/dom";
import { useLexicalComposerContext } from "@lexical/react/LexicalComposerContext";
import {
	$getSelection,
	$isRangeSelection,
	COMMAND_PRIORITY_LOW,
} from "lexical";
import { useCallback, useEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import FloatingMenu, {
	FloatingMenuCoordinates as FloatingMenuCoordinates,
} from "./FloatingMenu";
import { usePointerInteractions } from "./hooks/usePointerInteractions";
import {
	$insertList,
	$removeList,
	INSERT_ORDERED_LIST_COMMAND,
	INSERT_UNORDERED_LIST_COMMAND,
	REMOVE_LIST_COMMAND,
} from "@lexical/list";

const DOM_ELEMENT = document.body;

export function FloatingMenuPlugin() {
    const [editor] = useLexicalComposerContext();
	const [coordinates, setCoordinates] =
		useState<FloatingMenuCoordinates>(undefined);
    const ref = useRef<HTMLDivElement>(null);

	const { isPointerDown, isPointerReleased } = usePointerInteractions();

	const calculatePosition = useCallback(() => {
		const domSelection = getSelection();
		const domRange =
			domSelection?.rangeCount !== 0 && domSelection?.getRangeAt(0);

		if (!domRange || !ref.current || isPointerDown)
			return setCoordinates(undefined);

		computePosition(domRange, ref.current, { placement: "top" })
			.then(pos => {
				setCoordinates({ x: Math.max(0, pos.x), y: pos.y - 10 });
			})
			.catch(() => {
				setCoordinates(undefined);
			});
	}, [isPointerDown]);

	const $handleSelectionChange = useCallback(() => {
		if (
			editor.isComposing() ||
			editor.getRootElement() !== document.activeElement
		) {
			setCoordinates(undefined);
			return;
		}

		const selection = $getSelection();

		if (
			$isRangeSelection(selection) &&
			!selection.anchor.is(selection.focus)
		) {
			calculatePosition();
		} else {
			setCoordinates(undefined);
		}
	}, [editor, calculatePosition]);

	useEffect(() => {
		const unregisterListener = editor.registerUpdateListener(
			({ editorState }) => {
				editorState.read(() => $handleSelectionChange());
			},
		);
		return unregisterListener;
	}, [editor, $handleSelectionChange]);

	const show = coordinates !== undefined;

	useEffect(() => {
		if (!show && isPointerReleased) {
			editor.getEditorState().read(() => $handleSelectionChange());
		}
		// Adding show to the dependency array causes an issue if
		// a range selection is dismissed by navigating via arrow keys.
		// eslint-disable-next-line react-hooks/exhaustive-deps
	}, [isPointerReleased, $handleSelectionChange, editor]);

	useEffect(() => {
        // TODO: move to own plugin
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

	return createPortal(
		<FloatingMenu ref={ref} editor={editor} coordinates={coordinates} />,
		DOM_ELEMENT,
	);
}
