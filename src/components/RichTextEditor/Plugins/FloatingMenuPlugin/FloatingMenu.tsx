import { ForwardedRef, forwardRef, useEffect, useState } from "react";
import { useLexicalComposerContext } from "@lexical/react/LexicalComposerContext";
import {
	$getSelection,
	$isRangeSelection,
	BLUR_COMMAND,
	COMMAND_PRIORITY_LOW,
	FOCUS_COMMAND,
	LexicalEditor,
	RangeSelection,
} from "lexical";
import styles from "../../styles.module.css";
import { defaultButtons } from "./defaultButtons";
import Icon from "@mdi/react";

export type FloatingMenuCoordinates = { x: number; y: number } | null;

interface IProps {
	editor: ReturnType<typeof useLexicalComposerContext>[0];
	coordinates: FloatingMenuCoordinates;
	additionalFloatingMenuButtons?: IFloatingMenuButton[];
}

export interface IFloatingMenuButton {
	icon: string;
	name: string;
	title: string;
	onClick: (editor: LexicalEditor, isActive: boolean) => void;
	isActive: (selection: RangeSelection) => boolean;
}

function FloatingMenu(
	{ editor, coordinates, additionalFloatingMenuButtons }: IProps,
	ref: ForwardedRef<HTMLDivElement>,
) {
	const [state, setState] = useState<Record<string, boolean>>({});
	const [isEditorFocused, setIsEditorFocused] = useState(() => {
		return editor.getRootElement() == document.activeElement;
	});
	const [isFloatingMenuFocused, setIsFloatingMenuFocused] = useState(false);

	useEffect(() => {
		const unregisterUpdateListener = editor.registerUpdateListener(
			({ editorState }) => {
				editorState.read(() => {
					const selection = $getSelection();
					if (!$isRangeSelection(selection)) return;
					setIsEditorFocused(true);

					for (const command of defaultButtons) {
						state[command.name] = command.isActive(selection);
					}

					for (const command of additionalFloatingMenuButtons ?? []) {
						state[command.name] = command.isActive(selection);
					}
					setState(state);
				});
			},
		);

		const unregisterBlurListener = editor.registerCommand(
			BLUR_COMMAND,
			() => {
				setIsEditorFocused(false);
				return false;
			},
			COMMAND_PRIORITY_LOW,
		);

		const unregisterFocusListener = editor.registerCommand(
			FOCUS_COMMAND,
			() => {
				setIsEditorFocused(true);
				return false;
			},
			COMMAND_PRIORITY_LOW,
		);

		return () => {
			unregisterUpdateListener();
			unregisterBlurListener();
			unregisterFocusListener();
		};
	}, [editor, state, additionalFloatingMenuButtons]);

	const shouldShow =
		(isEditorFocused || isFloatingMenuFocused) && coordinates;

	// TODO: refactor(move command to own component)
	return (
		<div
			ref={ref}
			className={styles.floatingMenu}
			aria-hidden={!shouldShow}
			style={{
				top: `${coordinates?.y}px`,
				left: `${coordinates?.x}px`,
				visibility: shouldShow ? "visible" : "hidden",
				opacity: shouldShow ? 1 : 0,
			}}
			onFocus={() => setIsFloatingMenuFocused(true)}
			onBlur={() => setIsFloatingMenuFocused(false)}>
			{additionalFloatingMenuButtons?.map(current => (
				<button
					key={current.name}
					onClick={() => current.onClick(editor, state[current.name])}
					className={`transparent ${state[current.name] && styles.activeButton}`}
					title={current.title}
					aria-label={current.title}>
					<Icon path={current.icon} size={1} />
				</button>
			))}

			{additionalFloatingMenuButtons &&
				additionalFloatingMenuButtons.length > 0 && (
					<div className={styles.verticalBorder} />
				)}

			{defaultButtons.map(current => (
				<button
					key={current.name}
					onClick={() => current.onClick(editor, state[current.name])}
					className={`transparent ${state[current.name] && styles.activeButton}`}
					title={current.title}
					aria-label={current.title}>
					<Icon path={current.icon} size={1} />
				</button>
			))}
		</div>
	);
}

export default forwardRef(FloatingMenu);
