import styles from "./styles.module.css";
import Cell from "../../../../api/cells/entities/cell";
import { default as IncrementalReadingType } from "../../../../api/cells/valueObjects/incrementalReading";
import { useEffect, useState } from "react";
import ImportContainer from "./ImportContainer";
import { Icon } from "@mdi/react";
import { mdiBookOpenVariantOutline } from "@mdi/js";
import ReadDialog from "./ReadDialog";
import { getIncrementalReadingSchedule } from "../../../../api/incrementalReading/schedulingApi";
import IncrementalReadingSchedule from "../../../../api/incrementalReading/incrementalReadingSchedule";
import formatDueDate from "../../../../utils/formatDueDate";

interface Props {
	cell: Cell;
	onChange: (content: string) => void;
}

export default function IncrementalReading({ cell, onChange }: Props) {
	const [incrementalReading, setIncrementalReading] = useState(() => {
		return JSON.parse(cell.content) as IncrementalReadingType;
	});

	const [showReadDialog, setShowReadDialog] = useState(false);
	const [schedule, setSchedule] = useState<IncrementalReadingSchedule | null>(
		null,
	);

	useEffect(() => {
		// TODO: not updating after import/finishing reading
		void getIncrementalReadingSchedule(cell.id).then(setSchedule);
	}, [cell.id]);

	const handleChange = (
		updater: (
			current: IncrementalReadingType,
		) => Partial<IncrementalReadingType>,
	) => {
		setIncrementalReading(current => {
			const updated = { ...current, ...updater(current) };
			onChange(JSON.stringify(updated));
			return updated;
		});
	};

	if (incrementalReading.content === null) {
		return <ImportContainer onImport={ir => handleChange(() => ir)} />;
	}

	// TODO: add icon
	return (
		<>
			<div className={styles.verticalForm}>
				<input
					type="text"
					placeholder="Title"
					value={incrementalReading.title!}
					onChange={e =>
						handleChange(() => ({ title: e.target.value }))
					}
				/>
				{schedule && (
					<p className={`${styles.scheduleStatus}`}>
						{schedule.completed
							? "Completed reading"
							: `Due ${formatDueDate(schedule.nextReadingDate)}`}
					</p>
				)}
				<button
					className={`primary ${styles.rowButton}`}
					onClick={() => setShowReadDialog(true)}>
					<Icon path={mdiBookOpenVariantOutline} size={1} />
					<span>Read now</span>
				</button>
			</div>

			{showReadDialog && (
				<ReadDialog
					cellId={cell.id}
					incrementalReading={incrementalReading}
					onClose={() => setShowReadDialog(false)}
					onChange={handleChange}
				/>
			)}
		</>
	);
}
