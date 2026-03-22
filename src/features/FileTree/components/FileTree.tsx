import styles from "./styles.module.css";
import FileTreeItem from "./FileTreeItem.tsx";
import UiFolder from "../../../types/ui/uiFolder.ts";
import useAppDispatch from "../../../hooks/useAppDispatch.ts";
import {
	moveFile,
	moveFolder,
} from "../../../stores/fileSystem/fileSystemActions.ts";
import { DragDropProvider } from "@dnd-kit/react";
import {
	DragEndEvent,
	Feedback,
	PointerActivationConstraints,
	PointerSensor,
} from "@dnd-kit/dom";
import isMobile from "../../../utils/isMobile.ts";
import FileItemSourceData from "../types/fileItemSourceData.ts";
import {
	FILE_ITEM_SOURCE_DATA,
	FILE_ITEM_TARGET_DATA,
} from "../config/constants.ts";
import FileItemTargetData from "../types/fileItemTargetData.ts";

interface Props {
	folder: UiFolder;
}

function FileTree({ folder }: Props) {
	const dispatch = useAppDispatch();

	const handleDragEnd: DragEndEvent = event => {
		if (
			event.canceled ||
			event.operation.target?.type !== FILE_ITEM_TARGET_DATA ||
			event.operation.source?.type !== FILE_ITEM_SOURCE_DATA
		)
			return;

		const { id, isFolder } = event.operation.source
			.data as FileItemSourceData;
		const { folderId } = event.operation.target.data as FileItemTargetData;

		if (id === folderId) return;

		if (isFolder) {
			void dispatch(moveFolder(id, folderId));
		} else {
			void dispatch(moveFile(id, folderId));
		}
	};

	const sensorActivationConstraint = isMobile()
		? new PointerActivationConstraints.Delay({ value: 200, tolerance: 10 })
		: new PointerActivationConstraints.Distance({ value: 5 });

	return (
		<div className={styles.fileTreeContainer}>
			<DragDropProvider
				onDragEnd={handleDragEnd}
				plugins={defaults => [
					...defaults,
					Feedback.configure({ dropAnimation: null }),
				]}
				sensors={[
					PointerSensor.configure({
						activationConstraints: [sensorActivationConstraint],
					}),
				]}>
				<FileTreeItem
					fullPath=""
					folder={folder}
					id={folder.id}
					depth={0}
				/>
			</DragDropProvider>
		</div>
	);
}

export default FileTree;
