import { useCallback, useEffect, useState } from "react";
import { Icon } from "@mdi/react";
import { mdiCardsOutline } from "@mdi/js";
import styles from "./styles.module.css";
import { CallApiFn } from "../../../hooks/useApi";
import CellWithPendingExtractsDto from "../../../api/incrementalReading/dto/cellWithPendingExtractsDto";
import { getCellsWithPendingExtracts } from "../../../api/incrementalReading/api/incrementalReadingApi";
import ExtractsReviewDialog, {
	CellToReview,
} from "../../ExtractsReview/components/ExtractsReviewDialog";
import ExtractsQueueRow from "./ExtractsQueueRow";

interface Props {
	callApi: CallApiFn;
	reloadToken: number;
	onReload: () => void;
}

export default function ExtractsQueue({
	callApi,
	reloadToken,
	onReload,
}: Props) {
	const [cells, setCells] = useState<CellWithPendingExtractsDto[]>([]);
	const [reviewCells, setReviewCells] = useState<CellToReview[] | null>(null);

	const fetchCells = useCallback(async () => {
		await callApi(async () =>
			setCells((await getCellsWithPendingExtracts()) ?? []),
		);
	}, [callApi]);

	useEffect(() => {
		void fetchCells();
	}, [fetchCells, reloadToken]);

	const handleClose = () => {
		setReviewCells(null);
		onReload();
	};

	return (
		<>
			<div className={styles.box}>
				<div className={styles.headerWithAction}>
					<p>Pending extracts</p>
					{cells.length > 0 && (
						<button
							className={`transparent ${styles.headerButton}`}
							onClick={() =>
								setReviewCells(
									cells.map(cell => ({
										id: cell.cellId,
										title: cell.title,
									})),
								)
							}
							title="Go through all pending extracts in all files">
							<Icon path={mdiCardsOutline} size={1} />
							<span>All extracts</span>
						</button>
					)}
				</div>

				<div className={styles.mainContent}>
					{cells.length === 0 && (
						<div className={styles.row}>
							<p>No extracts to review right now.</p>
						</div>
					)}

					{cells.map(cell => (
						<ExtractsQueueRow
							key={cell.cellId}
							cell={cell}
							onClick={() =>
								setReviewCells([
									{ id: cell.cellId, title: cell.title },
								])
							}
						/>
					))}
				</div>
			</div>

			{reviewCells && (
				<ExtractsReviewDialog
					cells={reviewCells}
					onClose={handleClose}
				/>
			)}
		</>
	);
}
