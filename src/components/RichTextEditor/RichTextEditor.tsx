import { AnyExtension } from "@tiptap/react";
import styles from "./styles.module.css";
import { useState } from "react";
import {
	InitialConfigType,
	LexicalComposer,
} from "@lexical/react/LexicalComposer";
import { RichTextPlugin } from "@lexical/react/LexicalRichTextPlugin";
import { ContentEditable } from "@lexical/react/LexicalContentEditable";
import { LexicalErrorBoundary } from "@lexical/react/LexicalErrorBoundary";
import { HistoryPlugin } from "@lexical/react/LexicalHistoryPlugin";
import { AutoFocusPlugin } from "@lexical/react/LexicalAutoFocusPlugin";
import { FloatingMenuPlugin } from "./Plugins/FloatingMenuPlugin/FloatingMenuPlugin";
import { ListPlugin } from "@lexical/react/LexicalListPlugin";
import { ListItemNode, ListNode } from "@lexical/list";
import FocusBlurPlugin from "./Plugins/FocusBlurPlugin";
import { LexicalEditor } from "lexical";
import { IFloatingMenuButton } from "./Plugins/FloatingMenuPlugin/FloatingMenu";

// TODO:  image resizer

interface IProps {
	content: string;
	title?: string;
	extraExtensions?: AnyExtension[];
    // TODO: rename
	commands?: IFloatingMenuButton[];
	autofocus?: boolean;
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
			<div className={styles.innerEditor}>
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
	extraExtensions?: AnyExtension[];
	commands?: ICommand[];
	autofocus?: boolean;
	onChange: (html: string) => void;
	onFocus?: (editor: LexicalEditor) => void;
	onBlur?: () => void;
}

function TiptapEditor({
	content,
	extraExtensions,
	commands,
	autofocus,
	onChange,
	onFocus,
	onBlur,
}: TiptapEditorProps) {
	const initialConfig: InitialConfigType = {
		namespace: "BrainyEditor",
		onError: console.error,
		nodes: [ListNode, ListItemNode],
		theme: {
			text: {
				underline: styles.underline,
			},
		},
	};

	// TODO: cloze, testing, refactoring, styling (body is scrollable in long file), etc..
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
			{autofocus && <AutoFocusPlugin />}
			<FloatingMenuPlugin />
			<ListPlugin />
			<FocusBlurPlugin onFocus={onFocus} onBlur={onBlur} />
		</LexicalComposer>
	);
}

export default RichTextEditor;
