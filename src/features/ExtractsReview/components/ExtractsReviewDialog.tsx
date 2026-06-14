import { useCallback, useEffect, useState } from "react";
import Dialog from "../../../components/Dialog/Dialog";
import Form, { FormHeader, FormRows } from "../../../components/Form/Form";
import Tag from "../../../components/Tag/Tag";
import styles from "./styles.module.css";
import {
	getPendingExtractsWithContent,
	updateExtractStatus,
} from "../../../api/incrementalReading/api/extractsApi";
import useApi from "../../../hooks/useApi";
import { Icon } from "@mdi/react";
import {
	mdiCardsOutline,
	mdiExitToApp,
	mdiPlusBoxOutline,
	mdiSkipNextOutline,
} from "@mdi/js";
import TagRow from "../../../components/Tag/TagRow";
import Spinner from "../../../components/Spinner/Spinner";
import { PendingExtractDto } from "../../../api/incrementalReading/dto/pendingExtractDto";

export interface CellToReview {
	id: string;
	title: string;
}

interface Props {
	cells: CellToReview[];
	onClose: () => void;
}

export default function ExtractsReviewDialog({ cells, onClose }: Props) {
	const [cellIndex, setCellIndex] = useState(0);
	const [extracts, setExtracts] = useState<PendingExtractDto[]>([]);
	const [extractIndex, setExtractIndex] = useState(0);
	const { callApi, isSendingRequest: isLoading } = useApi();

	const loadExtracts = useCallback(
		async (idx: number) => {
			if (idx >= cells.length) {
				onClose();
				return;
			}
			await callApi(async () => {
				const pending = await getPendingExtractsWithContent(
					cells[idx].id,
				);
				if (pending.length === 0) {
					setCellIndex(idx + 1);
				} else {
					setExtracts(pending);
					setExtractIndex(0);
				}
			});
		},
		[cells, callApi, onClose],
	);

	useEffect(() => {
		void loadExtracts(cellIndex);
	}, [cellIndex, loadExtracts]);

	const advance = () => {
		if (extractIndex + 1 < extracts.length) {
			setExtractIndex(i => i + 1);
		} else if (cellIndex + 1 < cells.length) {
			setCellIndex(i => i + 1);
		} else {
			onClose();
		}
	};

	const handleDismiss = async () => {
		await callApi(() =>
			updateExtractStatus(extracts[extractIndex].id, "Dismissed"),
		);
		advance();
	};

	const handleAdd = async () => {
		await callApi(() =>
			updateExtractStatus(extracts[extractIndex].id, "Added"),
		);
		advance();
	};

	if (cells.length === 0 || extracts.length === 0) return;

	const currentCell = cells[cellIndex];
	const currentExtract = extracts[extractIndex];

	return (
		<Dialog focusTrap onHide={onClose} className={styles.dialog}>
			<Form
				onSubmit={e => {
					e.preventDefault();
					void handleAdd();
				}}
				className={styles.form}>
				<FormHeader icon={mdiCardsOutline} title={currentCell.title} />

				<TagRow>
					{extracts.length > 0 && (
						<Tag
							text={`${extractIndex + 1} / ${extracts.length}`}
							type="primary"
						/>
					)}
					{cells.length > 1 && (
						<Tag
							text={`Article ${cellIndex + 1} / ${cells.length}`}
						/>
					)}
				</TagRow>

				<FormRows
					rows={[
						{
							// TODO: rich text editor for cloze
							children: isLoading ? (
								<Spinner text="Loading..." />
							) : (
								<div
									dangerouslySetInnerHTML={{
										__html: currentExtract.innerHtml,
									}}
								/>
							),
						},
					]}
				/>

				<div className={styles.buttons}>
					<button
						className="transparent"
						type="button"
						onClick={onClose}>
						<Icon path={mdiExitToApp} size={1} />
						Exit
					</button>
					<button
						className="transparent"
						type="button"
						onClick={() => void handleDismiss()}
						disabled={isLoading || !currentExtract}>
						<Icon path={mdiSkipNextOutline} size={1} />
						Dismiss
					</button>
					<button
						className="primary"
						type="submit"
						disabled={isLoading || !currentExtract}>
						<Icon path={mdiPlusBoxOutline} size={1} />
						Add
					</button>
				</div>
			</Form>
		</Dialog>
	);
}
