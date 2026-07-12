import { useEffect, useMemo, useRef, useState } from "react";
import {
	AutoFocusExtension,
	ClickAfterLastBlockExtension,
	SelectBlockExtension,
	TabIndentationExtension,
} from "@lexical/extension";
import { CodeShikiExtension } from "@lexical/code-shiki";
import { $generateNodesFromDOM } from "@lexical/html";
import { HistoryExtension } from "@lexical/history";
import { ListExtension } from "@lexical/list";
import { TableExtension } from "@lexical/table";
import { ContentEditable } from "@lexical/react/LexicalContentEditable";
import { useLexicalComposerContext } from "@lexical/react/LexicalComposerContext";
import { LexicalExtensionComposer } from "@lexical/react/LexicalExtensionComposer";
import { RichTextExtension } from "@lexical/rich-text";
import {
	$createParagraphNode,
	$getRoot,
	$isDecoratorNode,
	$isElementNode,
	configExtension,
	defineExtension,
	HISTORY_MERGE_TAG,
	LexicalEditor,
} from "lexical";
import { Box, Text, Typography } from "@mantine/core";
import { DragPlugin } from "./plugins/DragPlugin";
import { SlashMenuPlugin } from "./plugins/SlashMenuPlugin";
import { EquationNode } from "./plugins/EquationPlugin/EquationNode";
import { EquationPlugin } from "./plugins/EquationPlugin/EquationPlugin";
import { HighlightNode } from "./plugins/HighlightPlugin/HighlightNode";
import { HighlightPlugin } from "./plugins/HighlightPlugin/HighlightPlugin";
import { HighlightCreatedPayload } from "./plugins/HighlightPlugin/highlightCommands";
import { ClozeHiddenNode } from "./plugins/ClozePlugin/ClozeHiddenNode";
import { ImageNode } from "./plugins/ImagePlugin/ImageNode";
import { ImagePlugin } from "./plugins/ImagePlugin/ImagePlugin";
import styles from "./Editor.module.css";

const blockTags = new Set([
	"P",
	"DIV",
	"H1",
	"H2",
	"H3",
	"H4",
	"H5",
	"H6",
	"UL",
	"OL",
	"LI",
	"BLOCKQUOTE",
	"TABLE",
	"TR",
	"TD",
	"TH",
	"PRE",
	"FIGURE",
]);

// Top-level blocks converted per editor update. Book-sized documents are
// split into chunks: the first chunk becomes the initial editor state so
// content is visible immediately, the rest stream in during idle time
// without blocking the main thread.
const CONTENT_CHUNK_SIZE = 200;

function htmlToChunks(html: string): Document[] {
	const parser = new DOMParser();
	const dom = parser.parseFromString(html, "text/html");

	// If the content has no block-level elements, wrap everything in a
	// single <p> so Lexical doesn't create a separate paragraph per
	// inline node (spans, links, etc.).
	const hasBlock = Array.from(dom.body.children).some(el =>
		blockTags.has(el.tagName),
	);
	if (!hasBlock && dom.body.childNodes.length > 0) {
		const p = dom.createElement("p");
		while (dom.body.firstChild) p.appendChild(dom.body.firstChild);
		dom.body.appendChild(p);
	}

	const chunks: Document[] = [];
	while (dom.body.firstChild) {
		const chunk = document.implementation.createHTMLDocument();
		for (let i = 0; i < CONTENT_CHUNK_SIZE && dom.body.firstChild; i++) {
			chunk.body.appendChild(chunk.adoptNode(dom.body.firstChild));
		}
		chunks.push(chunk);
	}
	return chunks;
}

function $appendChunk(editor: LexicalEditor, chunk: Document) {
	const nodes = $generateNodesFromDOM(editor, chunk);
	const root = $getRoot();

	// Used to avoid the following error:
	// Only element or decorator nodes can be inserted in to the root node.
	nodes.forEach(node => {
		if ($isElementNode(node) || $isDecoratorNode(node)) {
			root.append(node);
		} else {
			const textContent = node.getTextContent().trim();
			if (textContent !== "") {
				const paragraph = $createParagraphNode();
				paragraph.append(node);
				root.append(paragraph);
			}
		}
	});
}

const scheduleIdle: (cb: () => void) => number =
	typeof window.requestIdleCallback === "function"
		? cb => window.requestIdleCallback(cb)
		: cb => window.setTimeout(cb, 0);

const cancelIdle: (handle: number) => void =
	typeof window.cancelIdleCallback === "function"
		? handle => window.cancelIdleCallback(handle)
		: handle => window.clearTimeout(handle);

interface StreamContentPluginProps {
	chunks: Document[];
	nextChunkRef: React.RefObject<number>;
}

// Appends the remaining chunks (the first one is loaded as the initial
// editor state) one editor update at a time during idle periods. Updates
// are tagged history-merge so undo can't remove streamed-in content.
function StreamContentPlugin({
	chunks,
	nextChunkRef,
}: StreamContentPluginProps) {
	const [editor] = useLexicalComposerContext();

	useEffect(() => {
		let handle: number | null = null;

		const appendNext = () => {
			handle = null;
			if (nextChunkRef.current >= chunks.length) return;
			const chunk = chunks[nextChunkRef.current++];
			editor.update(() => $appendChunk(editor, chunk), {
				tag: HISTORY_MERGE_TAG,
			});
			handle = scheduleIdle(appendNext);
		};

		handle = scheduleIdle(appendNext);
		return () => {
			if (handle !== null) cancelIdle(handle);
		};
	}, [editor, chunks, nextChunkRef]);

	return null;
}

interface EditorProps {
	initialContent?: string;
	autoFocus?: boolean;
	children?: React.ReactNode;
	onHighlightCreated?: (payload: HighlightCreatedPayload) => void;
}

export default function Editor({
	initialContent,
	autoFocus = false,
	children,
	onHighlightCreated,
}: EditorProps) {
	const [anchorElem, setAnchorElem] = useState<HTMLElement | null>(null);

	const contentChunks = useMemo(
		() => (initialContent ? htmlToChunks(initialContent) : []),
		// eslint-disable-next-line react-hooks/exhaustive-deps -- only apply initialContent once, at editor creation
		[],
	);
	// Index of the next chunk to stream in; chunk 0 is the initial state.
	const nextChunkRef = useRef(1);

	const editorExtension = useMemo(
		() =>
			defineExtension({
				dependencies: [
					RichTextExtension,
					HistoryExtension,
					ListExtension,
					TableExtension,
					TabIndentationExtension,
					ClickAfterLastBlockExtension,
					SelectBlockExtension,
					CodeShikiExtension,
					configExtension(AutoFocusExtension, {
						defaultSelection: "rootStart",
						disabled: !autoFocus,
					}),
				],
				theme: {
					tableScrollableWrapper: styles["table-scrollable-wrapper"],
					text: {
						code: styles["inline-code"],
					},
					code: styles["code-block"],
				},
				name: "editor",
				namespace: "editor",
				nodes: [
					EquationNode,
					HighlightNode,
					ClozeHiddenNode,
					ImageNode,
				],
				$initialEditorState:
					contentChunks.length > 0
						? (editor: LexicalEditor) =>
								$appendChunk(editor, contentChunks[0])
						: undefined,
			}),
		// eslint-disable-next-line react-hooks/exhaustive-deps -- only apply initialContent once, at editor creation
		[],
	);

	return (
		<Typography>
			<LexicalExtensionComposer
				extension={editorExtension}
				contentEditable={null}>
				<Box className={styles.anchor} ref={setAnchorElem}>
					{/* Native scrolling instead of Mantine ScrollArea: its
					    resize-observed, table-layout viewport re-measures
					    book-sized content while scrolling. */}
					<Box h="100%" style={{ overflowY: "auto" }}>
						<ContentEditable
							spellCheck={false}
							className={styles["content-editable"]}
							aria-label="Rich text editor"
							aria-placeholder="Type '/' for commands..."
							placeholder={
								<Text className={styles.placeholder} c="dimmed">
									Type &apos;/&apos; for commands...
								</Text>
							}
						/>
					</Box>
					{contentChunks.length > 1 ? (
						<StreamContentPlugin
							chunks={contentChunks}
							nextChunkRef={nextChunkRef}
						/>
					) : null}
					<SlashMenuPlugin />
					<EquationPlugin />
					<ImagePlugin />
					<HighlightPlugin onHighlightCreated={onHighlightCreated} />
					{anchorElem ? <DragPlugin anchorElem={anchorElem} /> : null}
					{children}
				</Box>
			</LexicalExtensionComposer>
		</Typography>
	);
}
