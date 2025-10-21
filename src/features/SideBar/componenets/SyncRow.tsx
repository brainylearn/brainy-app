import Icon from "@mdi/react";
import styles from "./styles.module.css";
import { mdiSync } from "@mdi/js";
import { useState } from "react";
import Dialog from "../../../components/Dialog/Dialog";
import Spinner from "../../../components/Spinner/Spinner";
import { sync } from "../../../api/syncApi";
import useAppDispatch from "../../../hooks/useAppDispatch";
import { getReviewTreeFolderForRoot } from "../../../stores/fileSystem/fileSystemActions";
import errorToString from "../../../utils/errorToString";
import useGlobalKey from "../../../hooks/useGlobalKey";

export default function SyncRow() {
	const [isSyncing, setIsSyncing] = useState(false);
	const dispatch = useAppDispatch();

	const callSync = async () => {
		try {
			setIsSyncing(true);
			// TODO: refersh current file if user in file, also refresh home, etc...
			await sync();
			await dispatch(getReviewTreeFolderForRoot());
		} catch (e) {
			alert(errorToString(e));
			console.error(e);
		} finally {
			setIsSyncing(false);
		}
	};

	useGlobalKey(e => {
		if (e.ctrlKey && e.key.toLowerCase() === "y") {
			e.preventDefault();
			void callSync();
		}
	});

	return (
		<>
			{isSyncing && (
				<Dialog className={styles.syncBox}>
					<Spinner />
					<p>Please wait, syncing your data...</p>
				</Dialog>
			)}
			<button
				className={`${styles.row}`}
				title="Sync (Ctrl + Y)"
				onClick={() => void callSync()}>
				<Icon path={mdiSync} size="1em" />
				<p>Sync</p>
			</button>
		</>
	);
}
