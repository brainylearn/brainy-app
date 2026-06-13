import {
	$isDecoratorNode,
	$isElementNode,
	$isParagraphNode,
	$isTextNode,
	LexicalNode,
	PointType,
	RangeSelection,
	TextNode,
} from "lexical";
import { $isMarkNode, MarkNode } from "@lexical/mark";

/**
 * Wraps the selection in a new node. If the selection already contains one or
 * more nodes of type T, they are merged into a single new node. The first
 * found existing node is passed to `createNode` so callers can copy its
 * properties (e.g. an index).
 */
export function $wrapSelectionInNode<T extends MarkNode>(
	selection: RangeSelection,
	isNode: (node: LexicalNode) => node is T,
	createNode: (existingNode: T | undefined) => T,
): T {
	skipWhitespace(selection);
	const allNodes: LexicalNode[] = [];
	const existingNodesOfSameType = [];

	for (const node of selection.extract()) {
		let current = node.getParent();
		let addedNode = false;

		while (current !== null) {
			if (isNode(current)) {
				addedNode = true;
				existingNodesOfSameType.push(current);
				allNodes.push(current);
				break;
			}
			current = current.getParent();
		}

		if (!addedNode) allNodes.push(node);
	}

	let newNode: T | null =
		existingNodesOfSameType.length > 0
			? createNode(existingNodesOfSameType[0])
			: createNode(undefined);
	allNodes[0].insertBefore(newNode);

	let lastNode: T = newNode;

	for (const node of allNodes) {
		if (
			!$isMarkNode(node) &&
			!$isTextNode(node) &&
			($isElementNode(node) || $isDecoratorNode(node)) &&
			!node.isInline()
		) {
			// If the element type cannot be wrapped, stopping the last element
			// before the current element, and recreating new node of the type
			// we want later.
			if (newNode !== null) lastNode = newNode;
			newNode = null;

			continue;
		}

		if (newNode === null) {
			newNode = createNode(lastNode);
			node.insertBefore(newNode);
		}

		if (isNode(node)) {
			for (const child of node.getChildren()) {
				newNode.append(child);
			}
			node.remove();
		} else {
			newNode.append(node);
		}
	}

	return lastNode;
}

/**
 * Removes the selected content from its wrapping node(s) of type T. Handles
 * partial selections at boundaries (splitting the wrapper) and fully-contained
 * wrappers (unwrapping them entirely).
 */
export function $removeSelectionFromNode<T extends MarkNode>(
	selection: RangeSelection,
	isNode: (node: LexicalNode) => node is T,
	createNode: (existingNode: T) => T,
): void {
	const [startPoint, endPoint] = getStartAndEndPointForSelection(selection);
	const wrappers = $getNodesOfTypeFromSelection(selection, isNode);

	let didPassSelectionStart = false;
	let didPassSelectionEnd = false;

	const startPointNode = startPoint.getNode();
	const endPointNode = endPoint.getNode();

	for (const wrapper of wrappers) {
		const children = wrapper.getChildren();
		didPassSelectionStart =
			didPassSelectionStart ||
			startPointNode.is(wrapper) ||
			startPointNode.is(wrapper.getParent());
		didPassSelectionEnd = didPassSelectionEnd || endPointNode.is(wrapper);

		_removeSingleWrapper(
			wrapper,
			startPoint,
			endPoint,
			createNode,
			didPassSelectionStart,
			didPassSelectionEnd,
		);

		didPassSelectionStart =
			didPassSelectionStart || children.some(c => c.is(startPointNode));
		didPassSelectionEnd =
			didPassSelectionEnd || children.some(c => c.is(endPointNode));
	}
}

function _removeSingleWrapper<T extends MarkNode>(
	wrapper: T,
	startPoint: PointType,
	endPoint: PointType,
	createNode: (existingNode: T) => T,
	didPassSelectionStart: boolean,
	didPassSelectionEnd: boolean,
): void {
	const beforeSelectionNode = createNode(wrapper);
	const selectionNodes: LexicalNode[] = [];
	const afterSelectionNode = createNode(wrapper);

	let passedSelectionStart = didPassSelectionStart;
	let passedSelectionEnd = didPassSelectionEnd;

	for (const child of wrapper.getChildren()) {
		if (
			!passedSelectionStart &&
			child.is(startPoint.getNode()) &&
			!passedSelectionEnd &&
			child.is(endPoint.getNode())
		) {
			passedSelectionStart = true;
			passedSelectionEnd = true;

			if (!$isTextNode(child)) continue;

			// Calculating this value before splitting since splitting updates offsets.
			const startsAtBoundary = startPoint.offset === 0;

			const textParts = child.splitText(
				startPoint.offset,
				endPoint.offset,
			);

			// Selected the whole thing.
			if (textParts.length === 1) selectionNodes.push(textParts[0]);

			// Selection start or end at boundary but not both.
			if (textParts.length === 2) {
				if (startsAtBoundary) {
					// Selection start at the first boundary
					selectionNodes.push(textParts[0]);
					afterSelectionNode.append(textParts[1]);
				} else {
					// Selection ends at the end boundary.
					beforeSelectionNode.append(textParts[0]);
					selectionNodes.push(textParts[1]);
				}
			}

			// Selection is inside the text
			if (textParts.length > 2) {
				beforeSelectionNode.append(textParts[0]);
				selectionNodes.push(textParts[1]);
				afterSelectionNode.append(textParts[2]);
			}
		} else if (!passedSelectionStart) {
			if (child.is(startPoint.getNode())) {
				passedSelectionStart = true;
				if (!$isTextNode(child)) continue;

				const textNodes = child.splitText(startPoint.offset);

				// Selection starts in the middle of element.
				if (textNodes.length > 1) {
					beforeSelectionNode.append(textNodes[0]);
					selectionNodes.push(textNodes[1]);
				}
				// The selection contains the whole element.
				else if (textNodes.length === 1) {
					selectionNodes.push(textNodes[0]);
				}
			} else {
				beforeSelectionNode.append(child);
			}
		} else if (!passedSelectionEnd) {
			if (child.is(endPoint.getNode())) {
				passedSelectionEnd = true;
				if (!$isTextNode(child)) continue;

				const textNodes = child.splitText(endPoint.offset);

				// Selection ends in the middle of element.
				if (textNodes.length > 1) {
					selectionNodes.push(textNodes[0]);
					afterSelectionNode.append(textNodes[1]);
				}
				// The selection contains the whole element.
				else if (textNodes.length === 1) {
					selectionNodes.push(textNodes[0]);
				}
			} else {
				selectionNodes.push(child);
			}
		} else if (passedSelectionEnd) {
			afterSelectionNode.append(child);
		} else if (passedSelectionStart) {
			selectionNodes.push(child);
		}
	}

	if (!beforeSelectionNode.isEmpty())
		wrapper.insertBefore(beforeSelectionNode);
	if (!afterSelectionNode.isEmpty()) wrapper.insertAfter(afterSelectionNode);
	selectionNodes.forEach(node => wrapper.insertBefore(node));

	if (wrapper.isEmpty()) wrapper.remove();
}

export function $isSelectionInsideNode<T extends LexicalNode>(
	selection: RangeSelection,
	isNode: (node: LexicalNode) => node is T,
): boolean {
	const [startPoint, endPoint] = getStartAndEndPointForSelection(selection);
	const startNode = startPoint.getNode();
	const endNode = endPoint.getNode();

	let passedStart = false;

	for (const node of selection.getNodes()) {
		if (!passedStart) {
			if (node.is(startNode)) {
				passedStart = true;
			} else continue;
		}

		// TODO: this is not correct, one line paragraphs are marked as containg highlight even if not
		// No need to stop at paragraphs, their content is still iterated over!
		if ($isParagraphNode(node)) continue;

		let anyMatch = isNode(node);
		let current = node.getParent();
		while (current !== null) {
			anyMatch = anyMatch || isNode(current);
			current = current.getParent();
		}

		if (!anyMatch) {
			return false;
		}

		if (node.is(endNode)) break;
	}

	return true;
}

export function $getNodesOfTypeFromSelection<T extends LexicalNode>(
	selection: RangeSelection,
	isNode: (node: LexicalNode) => node is T,
): T[] {
	const nodes: T[] = [];

	for (const node of selection.getNodes()) {
		let current: LexicalNode | null = node.getParent();
		while (current !== null) {
			if (isNode(current)) {
				nodes.push(current);
				break;
			}
			current = current.getParent();
		}
	}
	return nodes;
}

function skipWhitespace(selection: RangeSelection) {
	const [startPoint, endPoint] = getStartAndEndPointForSelection(selection);

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
	let currentEndOffset = endPoint.offset - 1;

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
			const prevSibling = currentEndNode.getPreviousSibling();
			if (prevSibling) {
				currentEndNode = $isTextNode(prevSibling) ? prevSibling : null;
				currentEndOffset = prevSibling.getTextContent().length - 1;
			} else {
				break;
			}
		}
	}

	if (!currentStartNode || !currentEndNode) return;

	const isStartBeforeEnd =
		currentStartNode.isBefore(currentEndNode) ||
		(currentStartNode.is(currentEndNode) &&
			currentStartOffset < currentEndOffset);

	// If the selection is only whitespace the start will be after the end,
	// so skip any change.
	if (isStartBeforeEnd) {
		selection.setTextNodeRange(
			currentStartNode as TextNode,
			currentStartOffset,
			currentEndNode as TextNode,
			// Plus one to include the last non-whitespace character.
			currentEndOffset + 1,
		);
	}
}

function getStartAndEndPointForSelection(
	selection: RangeSelection,
): [PointType, PointType] {
	return selection.isBackward()
		? [selection.focus, selection.anchor]
		: [selection.anchor, selection.focus];
}
