import { useRef, useState } from "react";
import styles from "./styles.module.css";
import useOutsideClick from "../../hooks/useOutsideClick";
import useGlobalKey from "../../hooks/useGlobalKey";
import { mdiChevronDown, mdiChevronUp } from "@mdi/js";
import Icon from "@mdi/react";

// TODO: use it everywhere

export interface Option {
	label: string;
	value: string | null;
}

interface Props {
	options: Option[];
	value: string | null;
	containerClassName?: string;
	onChange: (value: string | null) => void;
}

// TODO: unit test
export default function Select({
	containerClassName,
	options,
	value,
	onChange,
}: Props) {
	const [isOpen, setIsOpen] = useState(false);
	const containerRef = useRef<HTMLDivElement | null>(null);

	const selectedLabel = options.find(option => option.value === value)?.label;

	useOutsideClick(containerRef as React.RefObject<HTMLElement>, () => {
		setIsOpen(false);
	});

	useGlobalKey(e => {
		if (e.key === "Escape") setIsOpen(false);
	});

	const handleOptionClick = (value: string | null) => {
		onChange(value);
		setIsOpen(false);
	};

	return (
		<div
			className={`${styles.container} ${containerClassName}`}
			ref={containerRef}>
			<button
				onClick={() => setIsOpen(!isOpen)}
				className={`transparent ${styles.dropDownButton}`}>
				<p>{selectedLabel}</p>
				<Icon path={isOpen ? mdiChevronUp : mdiChevronDown} size={1} />
			</button>

			{isOpen && (
				<div className={styles.options}>
					{options.map((option, i) => (
						<button
							key={option.value}
							className={`${option.value === value ? "primary" : "transparent"}`}
							onClick={() => handleOptionClick(option.value)}
							autoFocus={i === 0}>
							{option.label}
						</button>
					))}
				</div>
			)}
		</div>
	);
}
