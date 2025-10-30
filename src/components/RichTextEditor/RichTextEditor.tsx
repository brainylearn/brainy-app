import styles from "./styles.module.css";
import { JSX, useState } from "react";
import {
	InitialConfigType,
	LexicalComposer,
} from "@lexical/react/LexicalComposer";
import { RichTextPlugin } from "@lexical/react/LexicalRichTextPlugin";
import { ContentEditable } from "@lexical/react/LexicalContentEditable";
import { LexicalErrorBoundary } from "@lexical/react/LexicalErrorBoundary";
import { HistoryPlugin } from "@lexical/react/LexicalHistoryPlugin";
import { FloatingMenuPlugin } from "./Plugins/FloatingMenuPlugin/FloatingMenuPlugin";
import { ListPlugin } from "@lexical/react/LexicalListPlugin";
import { ListItemNode, ListNode } from "@lexical/list";
import FocusBlurPlugin from "./Plugins/FocusBlurPlugin";
import {
	LexicalEditor,
	$getRoot,
	EditorState,
	LexicalNode,
	Klass,
} from "lexical";
import { IFloatingMenuButton } from "./Plugins/FloatingMenuPlugin/FloatingMenu";
import ListCommandsPlugin from "./Plugins/ListCommandsPlugin";
import { $generateHtmlFromNodes, $generateNodesFromDOM } from "@lexical/html";
import { OnChangePlugin } from "@lexical/react/LexicalOnChangePlugin";
import AutoFocusPlugin from "./Plugins/AutoFocusPlugin";

// TODO:  image resizer

interface IProps {
	content: string;
	title?: string;
	extraNodes?: Klass<LexicalNode>[];
	plugins?: JSX.Element[];
	additionalFloatingMenuButtons?: IFloatingMenuButton[];
	autofocus?: boolean;
	// TODO: update comment
	/** TiptapEditor is slow on rendering, therefor showing a div element
	 * instead until there is a need to render the actual editor (e.g. user interaction).
	 */
	eagerLoadRichTextEditor: boolean;
	onChange: (html: string) => void;
	onFocus?: (editor: LexicalEditor) => void;
	onBlur?: () => void;
}

function RichTextEditor({ eagerLoadRichTextEditor, ...props }: IProps) {
	const [showEditor, setShowEditor] = useState(eagerLoadRichTextEditor);
	const [
		previousEagerLoadRichTextEditor,
		setPreviousEagerLoadRichTextEditor,
	] = useState<boolean | null>(null);
	if (previousEagerLoadRichTextEditor !== eagerLoadRichTextEditor) {
		setPreviousEagerLoadRichTextEditor(eagerLoadRichTextEditor);
		if (eagerLoadRichTextEditor) setShowEditor(true);
	}

	return (
		<>
			{props.title && <p className={styles.title}>{props.title}</p>}
			<div className={styles.container}>
				{showEditor && <TiptapEditor {...props} />}
				{!showEditor && (
					<div className={`${styles.editor}`}>
						<div
							tabIndex={0}
							dangerouslySetInnerHTML={{
								// Setting white space if content is empty so that the height is correct.
								__html: props.content
									? props.content
									: "&nbsp;",
							}}
							onMouseEnter={() => setShowEditor(true)}
							onFocus={() => setShowEditor(true)}
						/>
					</div>
				)}
			</div>
		</>
	);
}

interface TiptapEditorProps {
	content: string;
	title?: string;
	extraNodes?: Klass<LexicalNode>[];
	additionalFloatingMenuButtons?: IFloatingMenuButton[];
	plugins?: JSX.Element[];
	autofocus?: boolean;
	onChange: (html: string) => void;
	onFocus?: (editor: LexicalEditor) => void;
	onBlur?: () => void;
}

function TiptapEditor({
	content,
	extraNodes,
	additionalFloatingMenuButtons,
	autofocus,
	plugins,
	onChange,
	onFocus,
	onBlur,
}: TiptapEditorProps) {
	const initialConfig: InitialConfigType = {
		namespace: "BrainyEditor",
		onError: console.error,
		nodes: [ListNode, ListItemNode, ...(extraNodes ?? [])],
		theme: {
			text: {
				underline: "underline",
				bold: "bold",
				italic: "italic",
			},
		},
		editorState: editor => {
			const parser = new DOMParser();
			const dom = parser.parseFromString(content, "text/html");
			const nodes = $generateNodesFromDOM(editor, dom);
			$getRoot().append(...nodes);
		},
	};

	const handleChange = (editorState: EditorState, editor: LexicalEditor) => {
		editorState.read(() => {
			const html = $generateHtmlFromNodes(editor);
			onChange(html);
		});
	};

	// TODO: cloze, testing, refactoring, styling, etc..
	return (
		<LexicalComposer initialConfig={initialConfig}>
			<RichTextPlugin
				contentEditable={
					<ContentEditable
						className={styles.editor}
						aria-placeholder={"Enter some text..."}
						placeholder={<></>}
					/>
				}
				ErrorBoundary={LexicalErrorBoundary}
			/>
			<HistoryPlugin />
			<OnChangePlugin onChange={handleChange} />
			<FloatingMenuPlugin
				additionalFloatingMenuButtons={additionalFloatingMenuButtons}
			/>
            <AutoFocusPlugin autofocus={autofocus ?? false} />
			<ListPlugin />
			<ListCommandsPlugin />
			<FocusBlurPlugin onFocus={onFocus} onBlur={onBlur} />
			{plugins}
		</LexicalComposer>
	);
}

export default RichTextEditor;
