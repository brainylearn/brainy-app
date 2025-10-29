import { useLexicalComposerContext } from "@lexical/react/LexicalComposerContext";
import { $getSelection, $isRangeSelection } from "lexical";
import { useCallback, useEffect, useRef, useState } from "react";
import FloatingMenu, {
	FloatingMenuCoordinates as FloatingMenuCoordinates,
} from "./FloatingMenu";
import { usePointerInteractions } from "./hooks/usePointerInteractions";

export function FloatingMenuPlugin() {
	const [editor] = useLexicalComposerContext();
	const [coordinates, setCoordinates] =
		useState<FloatingMenuCoordinates>(null);
	const ref = useRef<HTMLDivElement>(null);

	const { isPointerDown, isPointerReleased } = usePointerInteractions();

	const calculatePosition = useCallback(() => {
		const domSelection = getSelection();
		const domRangeRect =
			domSelection?.rangeCount !== 0 &&
			domSelection?.getRangeAt(0)?.getBoundingClientRect();
		const editorRootElementRect = editor
			.getRootElement()
			?.getBoundingClientRect();

		if (
			!domRangeRect ||
			!ref.current ||
			isPointerDown ||
			!editorRootElementRect
		) {
			return setCoordinates(null);
		}

		const newCoordinates = {
			x:
				Math.max(
					0,
					domRangeRect.left -
						editorRootElementRect.left -
						ref.current.getBoundingClientRect().width / 2,
				),
			y: domRangeRect.top - editorRootElementRect.top - 10,
		};
		setCoordinates(newCoordinates);
	}, [editor, isPointerDown]);

	const $handleSelectionChange = useCallback(() => {
		if (
			editor.isComposing() ||
			editor.getRootElement() !== document.activeElement
		) {
			setCoordinates(null);
			return;
		}

		const selection = $getSelection();

		if (
			$isRangeSelection(selection) &&
			!selection.anchor.is(selection.focus)
		) {
			calculatePosition();
		} else {
			setCoordinates(null);
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

	const show = coordinates !== null;
	useEffect(() => {
		if (!show && isPointerReleased) {
			editor.getEditorState().read(() => $handleSelectionChange());
		}
		// Adding show to the dependency array causes an issue if
		// a range selection is dismissed by navigating via arrow keys.
		// eslint-disable-next-line react-hooks/exhaustive-deps
	}, [isPointerReleased, $handleSelectionChange, editor]);

	return <FloatingMenu ref={ref} editor={editor} coordinates={coordinates} />;
}
