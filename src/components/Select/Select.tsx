import { useEffect, useRef, useState } from "react";
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
	onChange: (value: string | null) => void;
}

// TODO: unit test
export default function Select({ options, value, onChange }: Props) {
	const [isOpen, setIsOpen] = useState(false);
	const optionsButtonRefs = useRef<(HTMLButtonElement | null)[]>([]);
	const containerRef = useRef<HTMLDivElement | null>(null);
	const dropDownRef = useRef<HTMLButtonElement | null>(null);
	const focusedIndex = useRef(0);

	const selectedLabel = options.find(option => option.value === value)?.label;

	useOutsideClick(containerRef as React.RefObject<HTMLElement>, () => {
		setIsOpen(false);
	});

	useGlobalKey(e => {
		if (e.key === "Escape") setIsOpen(false);
	});

	useEffect(() => {
		if (!isOpen && document.activeElement === document.body)
			dropDownRef.current?.focus();
	}, [isOpen]);

	const handleContainerKeyDown = (e: React.KeyboardEvent) => {
		let element;
		if (e.key === "ArrowDown") {
			element =
				optionsButtonRefs.current[
					(focusedIndex.current + 1) % options.length
				];
		} else if (e.key === "ArrowUp") {
			element =
				optionsButtonRefs.current[
					(focusedIndex.current + options.length - 1) % options.length
				];
		}

		if (element) {
			e.preventDefault();
			element.focus();
		}
	};

	const handleOptionClick = (value: string | null) => {
		onChange(value);
		setIsOpen(false);
	};

	return (
		<div className={`${styles.container}`} ref={containerRef}>
			<button
				onClick={() => setIsOpen(!isOpen)}
				ref={dropDownRef}
				className={`transparent ${styles.dropDownButton}`}>
				<p>{selectedLabel}</p>
				<Icon path={isOpen ? mdiChevronUp : mdiChevronDown} size={1} />
			</button>

			{isOpen && (
				<div
					className={styles.options}
					onKeyDown={handleContainerKeyDown}>
					{options.map((option, i) => (
						<button
							key={option.value}
							className={`${option.value === value ? "primary" : "transparent"}`}
							onClick={() => handleOptionClick(option.value)}
							autoFocus={i === 0}
							ref={el => {
								optionsButtonRefs.current[i] = el;
							}}
							onFocus={() => {
								focusedIndex.current = i;
							}}>
							{option.label}
						</button>
					))}
				</div>
			)}
		</div>
	);
}
