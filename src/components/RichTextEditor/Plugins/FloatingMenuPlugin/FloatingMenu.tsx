import { ForwardedRef, forwardRef, useEffect, useState } from "react";
import { useLexicalComposerContext } from "@lexical/react/LexicalComposerContext";
import { $getSelection, $isRangeSelection, BLUR_COMMAND, COMMAND_PRIORITY_LOW, FOCUS_COMMAND, LexicalEditor, RangeSelection } from "lexical";
import styles from "../../styles.module.css";
import { defaultButtons } from "./defaultButtons";
import Icon from "@mdi/react";

export type FloatingMenuCoordinates = { x: number; y: number } | undefined;

interface IProps {
	editor: ReturnType<typeof useLexicalComposerContext>[0];
	coordinates: FloatingMenuCoordinates;
}

export interface IFloatingMenuButton {
	icon: string;
	name: string;
	title: string;
	onClick: (editor: LexicalEditor, isActive: boolean) => void;
	isActive: (selection: RangeSelection) => boolean;
}

function FloatingMenu({editor, coordinates}: IProps, ref: ForwardedRef<HTMLInputElement>) {
    const [state, setState] = useState<Record<string, boolean>>({});
    const [isFocused, setIsFocused] = useState(false);

    useEffect(() => {
        const unregisterUpdateListener = editor.registerUpdateListener(
            ({ editorState }) => {
                editorState.read(() => {
                    const selection = $getSelection();
                    if (!$isRangeSelection(selection)) return;

                    for (const command of defaultButtons) {
                        state[command.name] = command.isActive(selection);
                    }
                    setState(state);
                });
            },
        );

		const unregisterBlurListener = editor.registerCommand(
			BLUR_COMMAND,
			() => {
                setIsFocused(false);
				return false;
			},
			COMMAND_PRIORITY_LOW,
		);

		const unregisterFocusListener = editor.registerCommand(
			FOCUS_COMMAND,
			() => {
                setIsFocused(true);
				return false;
			},
			COMMAND_PRIORITY_LOW,
		);

        return () => {
            unregisterUpdateListener();
            unregisterBlurListener();
            unregisterFocusListener();
        }
    }, [editor, state]);

    // TODO: hide on scroll
    const shouldShow = isFocused && coordinates;

    return (
        <div
            ref={ref}
            className={styles.floatingMenu}
            aria-hidden={!shouldShow}
            style={{
                position: "absolute",
                top: coordinates?.y,
                left: coordinates?.x,
                visibility: shouldShow ? "visible" : "hidden",
                opacity: shouldShow ? 1 : 0,
            }}>
            {defaultButtons.map(
                (
                    command,
                ) => (
                    <button
                        key={command.name}
                        onClick={() =>
                            command.onClick(editor, state[command.name])
                        }
                        className={`transparent ${state[command.name] && styles.activeButton}`}
                        title={command.title}
                        aria-label={command.title}>
                        <Icon path={command.icon} size={1} />
                    </button>
                ),
            )}
        </div>
    );
}

export default forwardRef(FloatingMenu);
