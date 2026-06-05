import IncrementalReading from "../../../../api/cells/valueObjects/incrementalReading";
import Dialog from "../../../../components/Dialog/Dialog";
import RichTextEditor from "../../../../components/RichTextEditor/RichTextEditor";
import { Icon } from "@mdi/react";
import styles from "./styles.module.css";
import { mdiCheck, mdiExitToApp, mdiMarker } from "@mdi/js";
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
	// TODO:
	// 1. Each highlights has a unique id
	// 2. Each change to highlights create new id
	// 3. When saving each cell extracts its highlights and save them into a table and delete no longer existing ones
	// 4. Add a new button to go through extracts (converting an extract to cloze removes it)
	// 5. Fix scheduling later (see claude)
	// 6. Add a button to convert the extract directly from editor

	return (
		<Dialog
			focusTrap
			className={styles.readDialog}
			onHide={onClose}
			fullScreenOnSmallDevices>
			<h2 className={styles.title}>{incrementalReading.title}</h2>
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
			<div className={styles.footer}>
				{/*TODO: change this to be a checkbox, and add priority dropwdown at start*/}
				<button
					className={`secondary ${styles.rowButton}`}
					onClick={onClose}>
					<Icon path={mdiCheck} size={1} />
					<span>Mark as completed</span>
				</button>
				<button
					className={`primary ${styles.rowButton}`}
					onClick={onClose}>
					<Icon path={mdiExitToApp} size={1} />
					<span>Stop for now</span>
				</button>
			</div>
		</Dialog>
	);
}
