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
	autofocus: boolean;
	onChange: (content: string) => void;
}

export default function IncrementalReading({
	cell,
	autofocus,
	onChange,
}: Props) {
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
	}, [cell.id, incrementalReading.source]);

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
		return (
			<ImportContainer
				autofocus={autofocus}
				onImport={ir => handleChange(() => ir)}
			/>
		);
	}

	// TODO: clicking does not put focus on the input (fix for import container too), see why and try to make a general solution
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
					autoFocus={autofocus}
				/>
				<button
					className={`primary ${styles.rowButton} ${styles.readButton}`}
					onClick={() => setShowReadDialog(true)}>
					<Icon path={mdiBookOpenVariantOutline} size={1} />
					<span className={styles.buttonContent}>
						<span>Read now</span>
						{schedule && (
							<span className={styles.buttonStatus}>
								{schedule.completed
									? "Already completed"
									: `Next read due is ${formatDueDate(schedule.nextReadingDate)}`}
							</span>
						)}
					</span>
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
