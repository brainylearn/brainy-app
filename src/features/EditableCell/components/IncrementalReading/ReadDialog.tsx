import IncrementalReading from "../../../../api/cells/valueObjects/incrementalReading";
import Dialog from "../../../../components/Dialog/Dialog";
import RichTextEditor from "../../../../components/RichTextEditor/RichTextEditor";
import { Icon } from "@mdi/react";
import styles from "./styles.module.css";
import { mdiDotsHorizontal, mdiExitToApp, mdiMarker } from "@mdi/js";
import {
	$getHighlightFromSelection,
	$isSelectionInsideHighlight,
	HighlightNode,
} from "./RichTextEditorPlugins/highlight/highlightNode";
import {
	HighlightPlugin,
	TOGGLE_HIGHLIGHT_NODE,
} from "./RichTextEditorPlugins/highlight/highlightPlugin";
import { $getSelection, $isRangeSelection, LexicalEditor } from "lexical";
import createDefaultCellDto from "../../../EditableCells/utils/createCreateCellRequestDto";
import Cell from "../../../../api/cells/entities/cell";
import useApi from "../../../../hooks/useApi";
import { createCell } from "../../../../api/cells/api/cellApi";

interface Props {
	incrementalReading: IncrementalReading;
	cell: Cell;
	onChange: (content: string) => void;
	onClose: () => void;
}

export default function ReadDialog({
	incrementalReading,
	cell,
	onChange,
	onClose,
}: Props) {
	// TODO: error
	const { errorMessage, callApi } = useApi();

	const handleClozeClick = (editor: LexicalEditor) => {
		editor.read(() => {
			const selection = $getSelection();
			if (!$isRangeSelection(selection)) return;
			const highlightNode = $getHighlightFromSelection(selection);
			if (!highlightNode) return;

			// TODO: create new cloze if none are found, and show editable cells dialog

			const element = editor.getElementByKey(highlightNode.getKey());
			const newCell = createDefaultCellDto(
				"Cloze",
				cell.fileId,
				1 /* TODO: cell index */,
			);
			newCell.content = element?.innerHTML ?? "";
			void callApi(async () => {
				// TODO: change content of the highlighted onde
				await createCell(newCell);
			});
		});
	};

	return (
		<Dialog focusTrap className={styles.readDialog} onHide={onClose}>
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
						{
							name: "Change cloze cell",
							title: "Change cloze cell",
							icon: mdiDotsHorizontal,
							onClick: handleClozeClick,
							isActive: () => false,
							isVisible: $isSelectionInsideHighlight,
						},
					]}
				/>
			</div>
			<div className={styles.footer}>
				<button
					className={`primary ${styles.rowButton}`}
					onClick={onClose}>
					<Icon path={mdiExitToApp} size={1} />
					<span>Close</span>
				</button>
			</div>
		</Dialog>
	);
}
