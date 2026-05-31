import {
	$isTextNode,
	LexicalNode,
	PointType,
	RangeSelection,
	TextNode,
} from "lexical";
import { $wrapSelectionInMarkNode, MarkNode } from "@lexical/mark";

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
	tag: string,
): T {
	skipWhitespace(selection);
	const allNodes: LexicalNode[] = [];
	let existingNode: T | undefined;

	for (const node of selection.extract()) {
		let current = node.getParent();
		let addedNode = false;

		while (current !== null) {
			if (isNode(current)) {
				addedNode = true;
				existingNode ??= current;
				allNodes.push(current);
				break;
			}
			current = current.getParent();
		}

		if (!addedNode) allNodes.push(node);
	}

	if (existingNode !== undefined) {
		const newNode = createNode(existingNode);
		allNodes[0].insertBefore(newNode);

		for (const node of allNodes) {
			if (isNode(node)) {
				for (const child of node.getChildren()) {
					newNode.append(child);
				}
				node.remove();
			} else {
				newNode.append(node);
			}
		}

		return newNode;
	} else {
		const newNode = createNode(undefined);
		$wrapSelectionInMarkNode(
			selection,
			selection.isBackward(),
			tag,
			() => newNode,
		);
		return newNode;
	}
}

/**
 * Removes the selected content from its wrapping node of type T. Content
 * before the selection stays in the original node; content after is moved into
 * a new node created by `createNode`.
 */
export function $removeSelectionFromNode<T extends MarkNode>(
	selection: RangeSelection,
	isNode: (node: LexicalNode) => node is T,
	createNode: (existingNode: T) => T,
): void {
	const wrapperNode = getWrappingNode(selection, isNode);
	if (!wrapperNode) return;

	const [startPoint, endPoint] =
		getStartEndAndEndPointForSelection(selection);
	const selectionNodes: LexicalNode[] = [];
	const afterNode = createNode(wrapperNode);

	let passedSelectionStart = startPoint.getNode().is(wrapperNode);
	let passedSelectionEnd = false;

	for (const child of wrapperNode.getChildren()) {
		if (passedSelectionEnd) {
			afterNode.append(child);
		} else if (
			!passedSelectionStart &&
			child.is(startPoint.getNode()) &&
			!passedSelectionEnd &&
			child.is(endPoint.getNode())
		) {
			passedSelectionStart = true;
			passedSelectionEnd = true;

			if ($isTextNode(child)) {
				const textNodes = child.splitText(
					startPoint.offset,
					endPoint.offset,
				);
				// Selected all text.
				if (textNodes.length === 1) selectionNodes.push(textNodes[0]);
				// Text after selection start.
				if (textNodes.length > 1) selectionNodes.push(textNodes[1]);
				// Text after selection end.
				if (textNodes.length > 2) afterNode.append(textNodes[2]);
			}
		} else if (!passedSelectionStart && child.is(startPoint.getNode())) {
			passedSelectionStart = true;

			if ($isTextNode(child)) {
				const textNodes = child.splitText(startPoint.offset);
				// Text after selection start.
				if (textNodes.length > 1) selectionNodes.push(textNodes[1]);
				// Selected everything.
				else if (
					textNodes.length === 1 &&
					startPoint.offset !== child.getTextContentSize()
				)
					selectionNodes.push(textNodes[0]);
			}
		} else if (!passedSelectionEnd && child.is(endPoint.getNode())) {
			passedSelectionEnd = true;

			if ($isTextNode(child)) {
				const textNodes = child.splitText(endPoint.offset);
				// Text before selection ends.
				if (textNodes.length > 0) selectionNodes.push(textNodes[0]);
				// Text after selection ends.
				if (textNodes.length > 1) afterNode.append(textNodes[1]);
			}
		} else if (passedSelectionStart) {
			selectionNodes.push(child);
		}
	}

	if (!afterNode.isEmpty()) wrapperNode.insertAfter(afterNode);
	selectionNodes.reverse().forEach(node => wrapperNode.insertAfter(node));
	if (wrapperNode.isEmpty()) wrapperNode.remove();
}

export function $isSelectionInsideNode<T extends LexicalNode>(
	selection: RangeSelection,
	isNode: (node: LexicalNode) => node is T,
): boolean {
	let allInside = true;
	for (const node of selection.getNodes()) {
		let anyMatch = false;
		let current = node.getParent();
		while (current !== null) {
			anyMatch = anyMatch || isNode(current);
			current = current.getParent();
		}
		allInside = allInside && anyMatch;
	}
	return allInside;
}

export function getWrappingNode<T extends LexicalNode>(
	selection: RangeSelection,
	isNode: (node: LexicalNode) => node is T,
): T | null {
	let found: T | null = null;
	for (const node of selection.getNodes()) {
		let current = node.getParent();
		while (current !== null) {
			if (isNode(current)) {
				found = current;
				break;
			}
			if (found) break;
			current = current.getParent();
		}
	}
	return found;
}

export function skipWhitespace(selection: RangeSelection) {
	const [startPoint, endPoint] =
		getStartEndAndEndPointForSelection(selection);

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

export function getStartEndAndEndPointForSelection(
	selection: RangeSelection,
): [PointType, PointType] {
	return selection.isBackward()
		? [selection.focus, selection.anchor]
		: [selection.anchor, selection.focus];
}
