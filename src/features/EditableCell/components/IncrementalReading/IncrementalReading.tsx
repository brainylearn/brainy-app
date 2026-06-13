import styles from "./styles.module.css";
import Cell from "../../../../api/cells/entities/cell";
import { default as IncrementalReadingType } from "../../../../api/cells/valueObjects/incrementalReading";
import { useState } from "react";
import ImportContainer from "./ImportContainer";
import { Icon } from "@mdi/react";
import { mdiBookOpenVariantOutline } from "@mdi/js";
import ReadDialog from "./ReadDialog";

interface Props {
	cell: Cell;
	onChange: (content: string) => void;
}

export default function IncrementalReading({ cell, onChange }: Props) {
	const [incrementalReading, setIncrementalReading] = useState(() => {
		return JSON.parse(cell.content) as IncrementalReadingType;
	});

	const [showReadDialog, setShowReadDialog] = useState(false);

	const handleChange = (newIncrementalReading: IncrementalReadingType) => {
		setIncrementalReading(newIncrementalReading);
		onChange(JSON.stringify(newIncrementalReading));
	};

	if (incrementalReading.content === null) {
		return <ImportContainer onImport={handleChange} />;
	}

	return (
		<>
			<div className={styles.verticalForm}>
				<input
					type="text"
					placeholder="Title"
					value={incrementalReading.title!}
					onChange={e =>
						handleChange({
							...incrementalReading,
							title: e.target.value,
						})
					}
				/>
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
					onChange={content => {
						handleChange({ ...incrementalReading, content });
					}}
					onChangePriority={priority => {
						handleChange({ ...incrementalReading, priority });
					}}
				/>
			)}
		</>
	);
}
