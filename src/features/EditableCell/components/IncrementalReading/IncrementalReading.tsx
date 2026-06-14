import styles from "./styles.module.css";
import Cell from "../../../../api/cells/entities/cell";
import { default as IncrementalReadingType } from "../../../../api/cells/valueObjects/incrementalReading";
import { useCallback, useEffect, useState } from "react";
import ImportContainer from "./ImportContainer";
import { Icon } from "@mdi/react";
import { mdiBookOpenVariantOutline } from "@mdi/js";
import ReadDialog from "./ReadDialog";
import { getIncrementalReadingSchedule } from "../../../../api/incrementalReading/schedulingApi";
import IncrementalReadingSchedule from "../../../../api/incrementalReading/incrementalReadingSchedule";
import formatDueDate from "../../../../utils/formatDueDate";
import useApi from "../../../../hooks/useApi";

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
	const [isImported, setIsImported] = useState(
		incrementalReading.content !== null,
	);
	const [showReadDialog, setShowReadDialog] = useState(false);
	const [schedule, setSchedule] = useState<IncrementalReadingSchedule | null>(
		null,
	);
	const { callApi, errorMessage } = useApi();

	const retrieveIncrementalReadingScehdule = useCallback(async () => {
		await callApi(async () => {
			const newSchedule = await getIncrementalReadingSchedule(cell.id);
			setSchedule(newSchedule);
		});
	}, [cell.id, callApi]);

	useEffect(() => {
		void retrieveIncrementalReadingScehdule();
	}, [retrieveIncrementalReadingScehdule, isImported]);

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

	if (!isImported) {
		return (
			<ImportContainer
				autofocus={autofocus}
				onImport={ir => {
					handleChange(() => ir);
					setIsImported(true);
				}}
			/>
		);
	}

	const handleCloseReadDialog = async () => {
		await retrieveIncrementalReadingScehdule();
		setShowReadDialog(false);
	};

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

				{errorMessage && (
					<p className={styles.errorMessage}>{errorMessage}</p>
				)}

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
									: `Due ${formatDueDate(schedule.nextReadingDate)}`}
							</span>
						)}
					</span>
				</button>
			</div>

			{showReadDialog && (
				<ReadDialog
					cellId={cell.id}
					incrementalReading={incrementalReading}
					onClose={() => void handleCloseReadDialog()}
					onChange={handleChange}
				/>
			)}
		</>
	);
}
