import { useCallback, useEffect, useRef, useState } from "react";
import Dialog from "../../../components/Dialog/Dialog";
import Form, { FormHeader, FormRows } from "../../../components/Form/Form";
import Tag from "../../../components/Tag/Tag";
import styles from "./styles.module.css";
import {
	createClozeFromExtract,
	getPendingExtractsWithContent,
	updateExtractStatus,
} from "../../../api/incrementalReading/api/extractsApi";
import useApi from "../../../hooks/useApi";
import { Icon } from "@mdi/react";
import {
	mdiCardsOutline,
	mdiExitToApp,
	mdiSkipNextCircleOutline,
} from "@mdi/js";
import TagRow from "../../../components/Tag/TagRow";
import Spinner from "../../../components/Spinner/Spinner";
import { PendingExtractDto } from "../../../api/incrementalReading/dto/pendingExtractDto";
import RichTextEditor from "../../../components/RichTextEditor/RichTextEditor";
import { ClozeFloatingMenuButtons } from "../../EditableCell/plugins/clozeFloatingMenuButtons";
import { ClozeNode } from "../../EditableCell/plugins/clozeNode";
import { ClozePlugin } from "../../EditableCell/plugins/clozePlugin";

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
	const editorContentRef = useRef<string>("");

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
					editorContentRef.current = pending[0].innerHtml;
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
			editorContentRef.current = extracts[extractIndex + 1].innerHtml;
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
			createClozeFromExtract(
				extracts[extractIndex].id,
				cells[cellIndex].id,
				editorContentRef.current,
			),
		);
		advance();
	};

	if (cells.length === 0 || extracts.length === 0) return;

	const currentCell = cells[cellIndex];
	const currentExtract = extracts[extractIndex];

	return (
		<Dialog
			focusTrap
			onHide={onClose}
			className={styles.dialog}
			fullScreenOnSmallDevices>
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
							text={`Highlight ${extractIndex + 1} / ${extracts.length}`}
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
					className={styles.rows}
					rows={[
						{
							className: styles.row,
							children: isLoading ? (
								<Spinner text="Loading..." />
							) : (
								<RichTextEditor
									title="Turn highlights into cloze cells"
									eagerLoadRichTextEditor
									content={currentExtract.innerHtml}
									onChange={content => {
										editorContentRef.current = content;
									}}
									extraNodes={[ClozeNode]}
									additionalFloatingMenuButtons={
										ClozeFloatingMenuButtons
									}
									plugins={[<ClozePlugin key={1} />]}
									containerClassName={styles.editor}
								/>
							),
						},
					]}
				/>

				<div className={styles.buttons}>
					<button
						className="transparent"
						type="button"
						onClick={onClose}
						title="Close without doing any modification">
						<Icon path={mdiExitToApp} size={1} />
						Exit
					</button>
					<button
						className="transparent"
						type="button"
						onClick={() => void handleDismiss()}
						disabled={isLoading || !currentExtract}
						title="Skip highlight">
						<Icon path={mdiSkipNextCircleOutline} size={1} />
						Dismiss
					</button>
					<button
						className="primary"
						type="submit"
						disabled={isLoading || !currentExtract}>
						<Icon path={mdiCardsOutline} size={1} />
						Save as cloze
					</button>
				</div>
			</Form>
		</Dialog>
	);
}
