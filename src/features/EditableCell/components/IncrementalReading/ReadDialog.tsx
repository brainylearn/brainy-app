import IncrementalReading from "../../../../api/cells/valueObjects/incrementalReading";
import Dialog from "../../../../components/Dialog/Dialog";
import RichTextEditor from "../../../../components/RichTextEditor/RichTextEditor";
import { Icon } from "@mdi/react";
import styles from "./styles.module.css";
import { mdiExitToApp } from "@mdi/js";

interface Props {
	incrementalReading: IncrementalReading;
	onChange: (content: string) => void;
	onClose: () => void;
}

export default function ReadDialog({
	incrementalReading,
	onChange,
	onClose,
}: Props) {
	return (
		<Dialog focusTrap className={styles.readDialog}>
			<div className={styles.header}>
				<h2>{incrementalReading.title}</h2>
				<button
					className={`primary ${styles.rowButton}`}
					onClick={onClose}>
					<Icon path={mdiExitToApp} size={1} />
					<span>Close</span>
				</button>
			</div>
			<div className={styles.readDialogBody}>
				<RichTextEditor
					content={incrementalReading.content!}
					eagerLoadRichTextEditor
					onChange={onChange}
				/>
			</div>
		</Dialog>
	);
}
