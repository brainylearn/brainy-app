import {
	ReviewTreeFile,
	ReviewTreeFolder,
} from "../api/fileSystem/dto/reviewTreeFolder";

function getFolderChildById(
	folder: ReviewTreeFolder,
	id: string,
): ReviewTreeFolder | ReviewTreeFile | null {
	let queue = [folder];

	while (queue.length > 0) {
		const folder = queue.pop()!;
		if (folder.id === id) return folder;

		for (const file of folder.files) {
			if (file.id === id) return file;
		}
		queue = [...queue, ...folder.subfolders];
	}

	return null;
}

export default getFolderChildById;
