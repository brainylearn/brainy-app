import styles from "./styles.module.css";
import Dialog from "../../../components/Dialog/Dialog";
import Form, { FormButtons, FormHeader } from "../../../components/Form/Form";
import { mdiTuneVariant } from "@mdi/js";

interface Props {
	onClose: () => void;
}

export default function FsrsDialog({ onClose }: Props) {
	const handleSubmit = (e: React.FormEvent) => {
		e.preventDefault();

		onClose();
	};

	return (
		<Dialog onHide={onClose} focusTrap className={styles.fsrsDialog}>
			<Form onSubmit={handleSubmit}>
				<FormHeader icon={mdiTuneVariant} title="FSRS Profile" />

				<FormButtons onClose={onClose} submitText="Save" />
			</Form>
		</Dialog>
	);
}
