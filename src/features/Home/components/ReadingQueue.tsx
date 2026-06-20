import { useCallback, useEffect, useState } from "react";
import styles from "./styles.module.css";
import { CallApiFn } from "../../../hooks/useApi";
import DueIncrementalReadingDto from "../../../api/incrementalReading/dto/dueIncrementalReadingDto";
import { getDueIncrementalReadings } from "../../../api/incrementalReading/api/incrementalReadingApi";
import {
	getCellById,
	updateCellsContents,
} from "../../../api/cells/api/cellApi";
import ReadDialog from "../../IncrementalReading/components/ReadDialog";
import IncrementalReading from "../../../api/cells/valueObjects/incrementalReading";
import ReadingQueueRow from "./ReadingQueueRow";

interface Props {
	callApi: CallApiFn;
}

interface ActiveReading {
	cellId: string;
	incrementalReading: IncrementalReading;
}

export default function ReadingQueue({ callApi }: Props) {
	const [readings, setReadings] = useState<DueIncrementalReadingDto[]>([]);
	const [activeReading, setActiveReading] = useState<ActiveReading | null>(
		null,
	);

	const fetchReadings = useCallback(async () => {
		await callApi(async () =>
			setReadings((await getDueIncrementalReadings()) ?? []),
		);
	}, [callApi]);

	useEffect(() => {
		void fetchReadings();
	}, [fetchReadings]);

	const handleOpen = (cellId: string) => {
		void callApi(async () => {
			const cell = await getCellById(cellId);
			const incrementalReading = JSON.parse(
				cell.content,
			) as IncrementalReading;
			setActiveReading({ cellId, incrementalReading });
		});
	};

	const handleChange = (
		updater: (current: IncrementalReading) => Partial<IncrementalReading>,
	) => {
		setActiveReading(current => {
			if (current === null) return current;
			return {
				...current,
				incrementalReading: {
					...current.incrementalReading,
					...updater(current.incrementalReading),
				},
			};
		});
	};

	const handleClose = () => {
		const current = activeReading;
		setActiveReading(null);
		if (current === null) return;

		// TODO: not working when clicking done
		void callApi(async () => {
			await updateCellsContents([
				{
					id: current.cellId,
					content: JSON.stringify(current.incrementalReading),
				},
			]);
			await fetchReadings();
		});
	};

	return (
		<>
			<div className={styles.box}>
				<div className={styles.header}>
					<p>Reading queue</p>
				</div>

				<div className={styles.mainContent}>
					{readings.length === 0 && (
						<div className={styles.row}>
							<p>No readings are due right now.</p>
						</div>
					)}

					{readings.map(reading => (
						<ReadingQueueRow
							key={reading.cellId}
							reading={reading}
							onClick={() => handleOpen(reading.cellId)}
						/>
					))}
				</div>
			</div>

			{activeReading && (
				<ReadDialog
					cellId={activeReading.cellId}
					incrementalReading={activeReading.incrementalReading}
					onChange={handleChange}
					onClose={handleClose}
				/>
			)}
		</>
	);
}
