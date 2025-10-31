import {
	$getSelection,
	$isRangeSelection,
	$isTextNode,
	COMMAND_PRIORITY_EDITOR,
	createCommand,
	DOMExportOutput,
	LexicalCommand,
	LexicalNode,
	NodeKey,
	RangeSelection,
	TextNode,
} from "lexical";
import { useLexicalComposerContext } from "@lexical/react/LexicalComposerContext";
import { useEffect } from "react";
import {
	$unwrapMarkNode,
	$wrapSelectionInMarkNode,
	MarkNode,
} from "@lexical/mark";

export class ClozeNode extends MarkNode {
	index: number;

	static getType(): string {
		return "cloze";
	}

	constructor(key: NodeKey | undefined = undefined) {
		super(undefined, key);
		this.index = 1;
	}

	canInsertTextBefore() {
		return true as unknown as false;
	}
	canInsertTextAfter() {
		return true as unknown as false;
	}

	static clone(node: ClozeNode): MarkNode {
		const clone = $createClozeNode(node.index);
		clone.__key = node.__key;
		return clone;
	}

	createDOM(): HTMLElement {
		const element = document.createElement("cloze");
		element.classList.add("cloze-mark");
		element.setAttribute("index", this.index.toString());
		return element;
	}

	updateDOM(): boolean {
		return false;
	}

	exportDOM(): DOMExportOutput {
		const element = document.createElement("cloze");
		element.classList.add("cloze-mark");
		element.setAttribute("index", this.index.toString());
		return { element };
	}

	excludeFromCopy() {
		return false;
	}

	static importDOM(): null {
		return {
			cloze: () => {
				return {
					conversion: (element: HTMLElement) => {
						const index = element.getAttribute("index");
						return { node: $createClozeNode(Number(index)) };
					},
					priority: 0,
				};
			},
			// This is necessary due to the return type of MarkNode super class.
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

export function $isAllSelectionInCloze(selection: RangeSelection): boolean {
	let allCloze = true;
	for (const node of selection.getNodes()) {
		let anyCloze = false;
		let current = node.getParent();
		while (current !== null) {
			anyCloze = anyCloze || $isClozeNode(current);
			current = current.getParent();
		}
		allCloze = allCloze && anyCloze;
	}

	return allCloze;
}

function skipWhitespace(selection: RangeSelection) {
	const [startPoint, endPoint] = selection.isBackward()
		? [selection.focus, selection.anchor]
		: [selection.anchor, selection.focus];

	let currentStartNode = $isTextNode(startPoint.getNode())
		? startPoint.getNode()
		: null;
	let currentStartOffset = startPoint.offset;

	while (currentStartNode) {
		const textContent = currentStartNode.getTextContent();

		if (currentStartOffset < textContent.length) {
			const char = textContent[currentStartOffset];

			if (/\s/.test(char)) {
				currentStartOffset++;
			} else {
				break;
			}
		} else {
			// Reached end of current node, try to move to next node
			const nextSibling = currentStartNode.getNextSibling();
			if (nextSibling) {
				currentStartNode = $isTextNode(nextSibling)
					? nextSibling
					: null;
				currentStartOffset = 0;
			} else {
				break;
			}
		}
	}

	let currentEndNode = $isTextNode(endPoint.getNode())
		? endPoint.getNode()
		: null;
	let currentEndOffset = endPoint.offset;

	while (currentEndNode) {
		const textContent = currentEndNode.getTextContent();

		if (currentEndOffset >= 0) {
			const char = textContent[currentEndOffset] ?? " ";

			if (/\s/.test(char)) {
				currentEndOffset--;
			} else {
				break;
			}
		} else {
			// Reached end of current node, try to move to next node
			const prevSibling = currentEndNode.getPreviousSibling();
			if (prevSibling) {
				currentEndNode = $isTextNode(prevSibling)
					? prevSibling
					: null;
				currentEndOffset = prevSibling.getTextContent().length;
			} else {
				break;
			}
		}
	}

	if (currentStartNode && currentEndNode && (currentStartNode.isBefore(currentEndNode) || (
        currentStartNode.is(currentEndNode) && currentStartOffset < currentEndOffset
                                                                                            )))
		selection.setTextNodeRange(
			currentStartNode as TextNode,
			currentStartOffset,
			currentEndNode as TextNode,
			currentEndOffset + 1,
		);
}

function $wrapAllSelectionWithCloze(selection: RangeSelection): ClozeNode {
	skipWhitespace(selection);
	const allNodes: LexicalNode[] = [];
	let clozeIndex: number | null = null;

	for (const node of selection.extract()) {
		let current = node.getParent();
		let added = false;

		while (current !== null) {
			if ($isClozeNode(current)) {
				added = true;
				clozeIndex ??= current.index;
				allNodes.push(current);
				break;
			}

			current = current.getParent();
		}

		if (!added) allNodes.push(node);
	}

	if (clozeIndex !== null) {
		const newClozeNode = $createClozeNode(clozeIndex);
		allNodes[0].insertBefore(newClozeNode);

		for (const node of allNodes) {
			if ($isClozeNode(node)) {
				const children = node.getChildren();
				for (const child of children) {
					newClozeNode.append(child);
				}
				node.remove();
			} else {
				newClozeNode.append(node);
			}
		}

		return newClozeNode;
	} else {
		const clozeNode = $createClozeNode(1);
		$wrapSelectionInMarkNode(
			selection,
			selection.isBackward(),
			"cloze",
			() => clozeNode,
		);
		return clozeNode;
	}
}

// TODO: move

export const TOGGLE_CLOZE_NODE: LexicalCommand<void> = createCommand();
export const INCREASE_CLOZE_GROUP_NUMBER: LexicalCommand<void> =
	createCommand();
export const DECREASE_CLOZE_GROUP_NUMBER: LexicalCommand<void> =
	createCommand();

// TODO: cannot have cursor before first character
export function ClozePlugin() {
	const [editor] = useLexicalComposerContext();

	useEffect(() => {
		if (!editor.hasNodes([ClozeNode])) {
			throw new Error("ClozeNode not registered on editor");
		}

		const unregisterToggleCloze = editor.registerCommand(
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

					if ($isAllSelectionInCloze(selection)) {
						for (const node of selection.extract()) {
							let current = node.getParent();
							while (current !== null) {
								if ($isClozeNode(current)) {
									// TODO: should split the text, maybe https://lexical.dev/docs/api/modules/lexical#splitnode
									$unwrapMarkNode(current);
									break;
								}
								current = current.getParent();
							}
						}
					} else {
						$wrapAllSelectionWithCloze(selection);
					}
				});
				return true;
			},
			COMMAND_PRIORITY_EDITOR,
		);

		const unregisterIncreaseGroupNumber = editor.registerCommand(
			INCREASE_CLOZE_GROUP_NUMBER,
			() => {
				editor.update(() => {
					const selection = $getSelection();

					if (
						!$isRangeSelection(selection) ||
						selection.isCollapsed()
					) {
						return;
					}

					const node = $wrapAllSelectionWithCloze(selection);
					node.index++;
				});
				return true;
			},
			COMMAND_PRIORITY_EDITOR,
		);

		const unregisterDecreaseGroupNumber = editor.registerCommand(
			DECREASE_CLOZE_GROUP_NUMBER,
			() => {
				editor.update(() => {
					const selection = $getSelection();

					if (
						!$isRangeSelection(selection) ||
						selection.isCollapsed()
					) {
						return;
					}

					const node = $wrapAllSelectionWithCloze(selection);
					node.index = Math.max(node.index - 1, 1);
				});
				return true;
			},
			COMMAND_PRIORITY_EDITOR,
		);

		return () => {
			unregisterToggleCloze();
			unregisterIncreaseGroupNumber();
			unregisterDecreaseGroupNumber();
		};
	}, [editor]);

	return null;
}
