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
	mdiExitToApp,
	mdiMarker,
	mdiTimerPauseOutline,
} from "@mdi/js";
import {
	$isSelectionInsideHighlight,
	HighlightNode,
} from "./RichTextEditorPlugins/highlight/highlightNode";
import {
	HIGHLIGHT_SHORTCUT_KEY,
	HighlightPlugin,
	TOGGLE_HIGHLIGHT_NODE,
} from "./RichTextEditorPlugins/highlight/highlightPlugin";
import { getModKeyLabel } from "../../../../utils/keyboardUtils";
import { useState } from "react";
import ScheduleLaterDialog from "./ScheduleLaterDialog";
import { scheduleIncrementalReadingLater } from "../../../../api/incrementalReading/api/incrementalReadingApi";

const priorityOptions: Option[] = [
	{ label: "High", value: "high" },
	{ label: "Normal", value: "normal" },
	{ label: "Low", value: "low" },
];

interface Props {
	cellId: string;
	incrementalReading: IncrementalReading;
	onChange: (
		updater: (current: IncrementalReading) => Partial<IncrementalReading>,
	) => void;
	onClose: () => void;
}

export default function ReadDialog({
	cellId,
	incrementalReading,
	onChange,
	onClose,
}: Props) {
	const [showScheduleLater, setShowScheduleLater] = useState(false);

	const handlePriorityChange = (value: string) => {
		const priority = value as IncrementalReadingPriority;
		onChange(() => ({ priority }));
	};

	const handleDone = () => {
		onChange(() => ({ completed: true }));
		onClose();
	};

	const handleScheduleLater = async (date: Date) => {
		await scheduleIncrementalReadingLater(cellId, date);
		setShowScheduleLater(false);
		onChange(() => ({ completed: false }));
		onClose();
	};

	// TODO:
	// 1. Add a button to convert the extract directly from editor (with a shortcut)
	// 2. Show the current article number under navigation buttons (the buttons should only be visible when going through the queue)
	// 3. Add sync support
	// 4. Add scheduling to the home screen (https://claude.ai/chat/81318681-44c1-403b-bf70-10d648415553)
	// 5. Auto remember scroll position by paragraph index

	return (
		<>
			<Dialog
				focusTrap
				className={styles.readDialog}
				onHide={onClose}
				fullScreenOnSmallDevices>
				<div className={styles.header}>
					<h2
						title={incrementalReading.title ?? ""}
						className={styles.title}>
						{incrementalReading.title}
					</h2>
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
						onChange={content => onChange(() => ({ content }))}
						extraNodes={[HighlightNode]}
						plugins={[<HighlightPlugin key={1} />]}
						additionalFloatingMenuButtons={[
							{
								name: "Toggle highlight",
								title: `Toggle highlight (${getModKeyLabel()}+Shift+${HIGHLIGHT_SHORTCUT_KEY.toUpperCase()})`,
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
						title="Close without rescheduling">
						<Icon path={mdiExitToApp} size={1} />
						<span>Exit</span>
					</button>
					<button
						className={`transparent ${styles.rowButton} ${styles.withBorder}`}
						onClick={handleDone}
						title="Mark as completed">
						<Icon path={mdiCheckCircleOutline} size={1} />
						<span>Done</span>
					</button>
					<button
						className={`primary ${styles.rowButton}`}
						onClick={() => setShowScheduleLater(true)}
						title="Continue reading later">
						<Icon path={mdiTimerPauseOutline} size={1} />
						<span>Later</span>
					</button>
				</div>
			</Dialog>

			{showScheduleLater && (
				<ScheduleLaterDialog
					onHide={() => setShowScheduleLater(false)}
					onSchedule={date => void handleScheduleLater(date)}
				/>
			)}
		</>
	);
}
