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
import IsMobile from "../../../utils/isMobile.ts";

interface Props {
	folder: UiFolder;
}

function FileTree({ folder }: Props) {
	const dispatch = useAppDispatch();

	const handleDragEnd: DragEndEvent = event => {
		if (
			event.canceled ||
			!event.operation.source ||
			!event.operation.target
		)
			return;

		// TODO: types
		const { id, isFolder } = event.operation.source.data as {
			id: string;
			isFolder: boolean;
		};
		const { folderId } = event.operation.target.data as {
			folderId: string;
		};

		if (id === folderId) return;

		if (isFolder) {
			void dispatch(moveFolder(id, folderId));
		} else {
			void dispatch(moveFile(id, folderId));
		}
	};

	const sensorActivationConstraint = IsMobile()
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
				<FileTreeItem fullPath="" folder={folder} id={folder.id} />
			</DragDropProvider>
		</div>
	);
}

export default FileTree;
