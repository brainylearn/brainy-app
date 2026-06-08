import IncrementalReading, {
	IncrementalReadingPriority,
} from "../../../../api/cells/valueObjects/incrementalReading";
import Dialog from "../../../../components/Dialog/Dialog";
import RichTextEditor from "../../../../components/RichTextEditor/RichTextEditor";
import Select, { Option } from "../../../../components/Select/Select";
import { Icon } from "@mdi/react";
import styles from "./styles.module.css";
import {
	mdiCheckCircleOutline,
	mdiMarker,
	mdiTimerPauseOutline,
} from "@mdi/js";
import {
	$isSelectionInsideHighlight,
	HighlightNode,
} from "./RichTextEditorPlugins/highlight/highlightNode";
import {
	HighlightPlugin,
	TOGGLE_HIGHLIGHT_NODE,
} from "./RichTextEditorPlugins/highlight/highlightPlugin";

const priorityOptions: Option[] = [
	{ label: "High", value: "high" },
	{ label: "Normal", value: "normal" },
	{ label: "Low", value: "low" },
];

interface Props {
	incrementalReading: IncrementalReading;
	onChange: (content: string) => void;
	onChangePriority: (priority: IncrementalReadingPriority) => void;
	onClose: () => void;
}

export default function ReadDialog({
	incrementalReading,
	onChange,
	onChangePriority,
	onClose,
}: Props) {
	const handlePriorityChange = (value: string) => {
		const priority = value as IncrementalReadingPriority;
		onChangePriority(priority);
	};
	// TODO:
	// 1. Save incremental readings into their own table and remember to remove them when the cell is removed
	// 2. Implement the scheduling for them as described in https://claude.ai/chat/81318681-44c1-403b-bf70-10d648415553
	// 3. When saving each cell extracts its highlights and save them into a table and delete no longer existing ones
	// 4. Add a new button to go through extracts (converting an extract to cloze removes it)
	// 5. Fix scheduling later (see claude), extracts have their own scheduling, and reading its own
	// 6. Add a button to convert the extract directly from editor

	return (
		<Dialog
			focusTrap
			className={styles.readDialog}
			onHide={onClose}
			fullScreenOnSmallDevices>
			<div className={styles.header}>
				<div className={styles.titleSection}>
					<h2>{incrementalReading.title}</h2>
					<p className={`dimmed ${styles.source}`}>
						{incrementalReading.source.url}
					</p>
				</div>
				<Select
					containerClassName={styles.select}
					options={priorityOptions}
					currentValue={incrementalReading.priority}
					onChangeValue={handlePriorityChange}
				/>
			</div>
			<div className={styles.readDialogBody}>
				<RichTextEditor
					content={incrementalReading.content!}
					containerClassName={styles.richTextEditor}
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
				<button
					className={`transparent ${styles.rowButton}`}
					onClick={onClose}>
					<Icon path={mdiCheckCircleOutline} size={1} />
					<span>Mark complete</span>
				</button>
				<button
					className={`primary ${styles.rowButton}`}
					onClick={onClose}>
					<Icon path={mdiTimerPauseOutline} size={1} />
					<span>Done for now</span>
				</button>
			</div>
		</Dialog>
	);
}
