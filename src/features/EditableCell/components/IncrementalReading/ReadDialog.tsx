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
	mdiChevronLeft,
	mdiChevronRight,
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
	// 1. Implement the scheduling for them as described in https://claude.ai/chat/81318681-44c1-403b-bf70-10d648415553
	// 2. Add a new button to go through extracts (converting an extract to cloze removes it)
	// 3. Add a button to convert the extract directly from editor
	// 4. Show the current article number under navigation buttons (the buttons should only be visible when going through the queue)
	// 5. Add sync support

	return (
		<Dialog
			focusTrap
			className={styles.readDialog}
			onHide={onClose}
			fullScreenOnSmallDevices>
			<div className={styles.header}>
				<div className={styles.titleSection}>
					<h2 title={incrementalReading.title ?? ""}>
						{incrementalReading.title}
					</h2>
					<p
						className={`dimmed ${styles.source}`}
						title={incrementalReading.source.url}>
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
				<div className={styles.navButtons}>
					<button
						className={`transparent ${styles.navButton}`}
						title="Previous reading">
						<Icon path={mdiChevronLeft} size={1} />
					</button>
					<button
						className={`transparent ${styles.navButton}`}
						title="Next reading">
						<Icon path={mdiChevronRight} size={1} />
					</button>
				</div>
				<button
					className={`transparent ${styles.rowButton} ${styles.withBorder}`}
					onClick={onClose}
					title="Mark as completed">
					<Icon path={mdiCheckCircleOutline} size={1} />
					<span>Done</span>
				</button>
				<button
					className={`primary ${styles.rowButton}`}
					onClick={onClose}
					title="Continue reading later">
					<Icon path={mdiTimerPauseOutline} size={1} />
					<span>Later</span>
				</button>
			</div>
		</Dialog>
	);
}
