import { mergeAttributes, Mark } from "@tiptap/core";
import { clozeMarkName } from "../config/constants";
import {
	$getSelection,
	$isRangeSelection,
	COMMAND_PRIORITY_EDITOR,
	createCommand,
	DOMConversionMap,
	DOMExportOutput,
	LexicalCommand,
	LexicalNode,
    NodeKey,
} from "lexical";
import { useLexicalComposerContext } from "@lexical/react/LexicalComposerContext";
import { useEffect } from "react";
import { $wrapSelectionInMarkNode, MarkNode } from "@lexical/mark";

// TODO: rename class
export class ClozeNode extends MarkNode {
    index: number;

	static getType(): string {
		return "cloze";
	}

    constructor(key: NodeKey | undefined = undefined) {
        super(undefined, key);
        this.index = 1;
    }

	createDOM(): HTMLElement {
		const dom = document.createElement("cloze");
		dom.classList.add("cloze-mark");
		dom.setAttribute("index", this.index.toString());
		return dom;
	}

	updateDOM(): boolean {
		return false;
	}

    exportDOM(): DOMExportOutput {
		const element = document.createElement("cloze");
		element.classList.add("cloze-mark");
		element.setAttribute("index", "1");
		return { element };
    }

      excludeFromCopy(destination: 'clone' | 'html') {
        return destination !== 'html'; // Include in HTML export
      }

    static importDOM(): null {
        return {
            cloze: () => {
                return {
                    conversion: (element: HTMLElement) => {
                        const index = element.getAttribute('index');
                        return { node: $createClozeNode(Number(index)) };
                    },
                    priority: 0,
                };
            },
        // This is necessary due to the return type of marknode super class.
        } as unknown as null;
    }
}

export function $createClozeNode(index: number): ClozeNode {
	const node = new ClozeNode();
    node.index = index;
    return node;
}

export function $isClozeNode(
	node: LexicalNode | null | undefined,
): node is ClozeNode {
	return node instanceof ClozeNode;
}

// TODO: move

export const TOGGLE_CLOZE_NODE: LexicalCommand<void> = createCommand();

export function ClozePlugin() {
	const [editor] = useLexicalComposerContext();

	useEffect(() => {
		if (!editor.hasNodes([ClozeNode])) {
			throw new Error("ClozeNode not registered on editor");
		}

		return editor.registerCommand(
			TOGGLE_CLOZE_NODE,
			() => {
				editor.update(() => {
					const selection = $getSelection();
					if (
						!$isRangeSelection(selection) ||
						selection.isCollapsed()
					) {
						return;
					}

					$wrapSelectionInMarkNode(
						selection,
						selection.isBackward(),
						"cloze-made",
						() => $createClozeNode(1),
					);
				});
				return true;
			},
			COMMAND_PRIORITY_EDITOR,
		);
	}, [editor]);

	return null;
}

declare module "@tiptap/core" {
	interface Commands<ReturnType> {
		customExtension: {
			toggleCloze: (index: number) => ReturnType;
			increaseClozeIndex: () => ReturnType;
			decreaseClozeIndex: () => ReturnType;
		};
	}
}

const clozeMark = Mark.create({
	name: clozeMarkName,

	parseHTML() {
		return [{ tag: clozeMarkName }];
	},

	addAttributes() {
		return {
			index: {
				renderHTML(attributes) {
					return {
						index: attributes.index as number,
					};
				},
			},
		};
	},

	renderHTML({ HTMLAttributes }) {
		return [
			"cloze",
			mergeAttributes(HTMLAttributes, {
				class: "cloze-mark",
			}),
			0,
		];
	},

	addCommands() {
		return {
			toggleCloze:
				index =>
				({ commands, editor }) => {
					if (editor.isActive(clozeMarkName)) {
						return commands.unsetMark(clozeMarkName);
					}

					const { from: selectionStart, to: selectionEnd } =
						editor.state.selection;
					const text = editor.getText();
					if (text.trim() === "") return true;

					let newSelectionStart = selectionStart;
					let newSelectionEnd = selectionEnd;

					// Removing extra whitespace at start.
					while (
						editor.state.doc
							.textBetween(newSelectionStart, newSelectionEnd)
							.endsWith(" ")
					) {
						newSelectionEnd--;
					}

					// Removing extra whitespace at end.
					while (
						editor.state.doc
							.textBetween(newSelectionStart, newSelectionEnd)
							.startsWith(" ")
					) {
						newSelectionStart++;
					}

					commands.setTextSelection({
						from: newSelectionStart,
						to: newSelectionEnd,
					});
					commands.unsetAllMarks();
					return commands.setMark(clozeMarkName, { index });
				},
			increaseClozeIndex:
				() =>
				({ commands, editor }) => {
					if (!editor.isActive(clozeMarkName)) return true;
					commands.extendMarkRange(clozeMarkName);
					return commands.updateAttributes(clozeMarkName, {
						index:
							(editor.getAttributes(clozeMarkName)
								.index as number) + 1,
					});
				},
			decreaseClozeIndex:
				() =>
				({ commands, editor }) => {
					if (!editor.isActive(clozeMarkName)) return true;
					commands.extendMarkRange(clozeMarkName);
					return commands.updateAttributes(clozeMarkName, {
						index: Math.max(
							1,
							(editor.getAttributes(clozeMarkName)
								.index as number) - 1,
						),
					});
				},
		};
	},
});

export default clozeMark;
