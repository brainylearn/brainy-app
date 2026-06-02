import { DOMExportOutput, LexicalNode, NodeKey, RangeSelection } from "lexical";
import { MarkNode } from "@lexical/mark";
import {
	$getNodesOfTypeFromSelection,
	$isSelectionInsideNode,
} from "../../../../../../components/RichTextEditor/Plugins/utils/selectionWrapUtils";

const HIGHLIGHT_CSS_CLASS_NAME = "highlight-node";
const HIGHLIGHT_EXTRACT_ID_ATTRIBUTE_NAME = "extract-ids";
export const HIGHLIGHT_TAG_NAME = "highlight";

export class HighlightNode extends MarkNode {
	extractId: string;

	constructor(key: NodeKey | undefined = undefined) {
		super(undefined, key);
		this.extractId = "";
	}

	canInsertTextBefore() {
		return true as unknown as false;
	}

	canInsertTextAfter() {
		return true as unknown as false;
	}

	static clone(node: HighlightNode): MarkNode {
		const clone = $createHighlightNode(node.extractId);
		clone.__key = node.__key;
		return clone;
	}

	createDOM(): HTMLElement {
		const element = document.createElement(HIGHLIGHT_TAG_NAME);
		element.classList.add(HIGHLIGHT_CSS_CLASS_NAME);
		element.setAttribute(
			HIGHLIGHT_EXTRACT_ID_ATTRIBUTE_NAME,
			JSON.stringify(this.extractId),
		);
		return element;
	}

	updateDOM(): boolean {
		return false;
	}

	excludeFromCopy() {
		return false;
	}

	exportDOM(): DOMExportOutput {
		const element = document.createElement(HIGHLIGHT_TAG_NAME);
		element.classList.add(HIGHLIGHT_CSS_CLASS_NAME);
		element.setAttribute(
			HIGHLIGHT_EXTRACT_ID_ATTRIBUTE_NAME,
			JSON.stringify(this.extractId),
		);
		return { element };
	}

	static importDOM(): null {
		return {
			highlight: () => {
				return {
					conversion: (element: HTMLElement) => {
						const raw = element.getAttribute(
							HIGHLIGHT_EXTRACT_ID_ATTRIBUTE_NAME,
						);
						const extractId: string = raw
							? (JSON.parse(raw) as string)
							: "";
						return { node: $createHighlightNode(extractId) };
					},
					priority: 0,
				};
			},
			// This is necessary due to the return type of MarkNode super class.
		} as unknown as null;
	}

	static getType(): string {
		return HIGHLIGHT_TAG_NAME;
	}
}

// TODO: default value
export function $createHighlightNode(extractId = ""): HighlightNode {
	const node = new HighlightNode();
	node.extractId = extractId;
	return node;
}

export function $isHighlightNode(
	node: LexicalNode | null | undefined,
): node is HighlightNode {
	return node instanceof HighlightNode;
}

export function $isSelectionInsideHighlight(
	selection: RangeSelection,
): boolean {
	return $isSelectionInsideNode(selection, $isHighlightNode);
}

export function $getHighlightFromSelection(selection: RangeSelection) {
	return $getNodesOfTypeFromSelection(selection, $isHighlightNode);
}
