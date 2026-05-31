import IncrementalReading from "../../../../api/cells/valueObjects/incrementalReading";
import Dialog from "../../../../components/Dialog/Dialog";
import RichTextEditor from "../../../../components/RichTextEditor/RichTextEditor";
import { Icon } from "@mdi/react";
import styles from "./styles.module.css";
import { mdiExitToApp, mdiMarker } from "@mdi/js";
import {
	$isSelectionInsideHighlight,
	HighlightNode,
} from "./RichTextEditorPlugins/highlight/highlightNode";
import {
	HighlightPlugin,
	TOGGLE_HIGHLIGHT_NODE,
} from "./RichTextEditorPlugins/highlight/highlightPlugin";

interface Props {
	incrementalReading: IncrementalReading;
	onChange: (content: string) => void;
	onClose: () => void;
}

export default function ReadDialog({
	incrementalReading,
	onChange,
	onClose,
}: Props) {
	return (
		<Dialog focusTrap className={styles.readDialog}>
			<div className={styles.header}>
				<h2>{incrementalReading.title}</h2>
				<button
					className={`primary ${styles.rowButton}`}
					onClick={onClose}>
					<Icon path={mdiExitToApp} size={1} />
					<span>Close</span>
				</button>
			</div>
			<div className={styles.readDialogBody}>
				<RichTextEditor
					content={incrementalReading.content!}
					eagerLoadRichTextEditor
					onChange={onChange}
					extraNodes={[HighlightNode]}
					plugins={[<HighlightPlugin key={1} />]}
					additionalFloatingMenuButtons={[
						{
							name: "Toggle highlight",
							title: "Toggle highlight",
							icon: mdiMarker,
							onClick: editor =>
								editor.dispatchCommand(
									TOGGLE_HIGHLIGHT_NODE,
									undefined,
								),
							isActive: $isSelectionInsideHighlight,
						},
					]}
				/>
			</div>
		</Dialog>
	);
}
