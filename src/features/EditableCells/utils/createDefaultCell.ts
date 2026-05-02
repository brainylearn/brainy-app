import Cell, { CellType } from "../../../api/cells/entities/cell";
import FlashCard from "../../../api/cells/valueObjects/flashCard";
import TrueFalse from "../../../api/cells/valueObjects/trueFalse";

function createDefaultCell(cellType: CellType, fileId: string, index: number) {
	const cell: Cell = {
		id: "",
		fileId,
		content: "",
		searchableContent: "",
		cellType,
		index,
		repetitions: [],
	};

	switch (cellType) {
		case "FlashCard":
			cell.content = JSON.stringify({
				question: "",
				answer: "",
			} as FlashCard);
			break;
		case "TrueFalse":
			cell.content = JSON.stringify({
				question: "",
				isTrue: true,
			} as TrueFalse);
			break;
		case "Note":
		case "Cloze":
			break;
	}
	return cell;
}

export default createDefaultCell;
