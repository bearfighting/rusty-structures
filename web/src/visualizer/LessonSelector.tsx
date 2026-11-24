import { availableLessons, lessonCatalog } from './catalog';

type Props = { selectedId?: string; currentLessonId?: string };

export default function LessonSelector({ selectedId, currentLessonId }: Props) {
  const activeId = selectedId ?? currentLessonId ?? 'binary-search-tree';
  return <div className="lesson-selector">
    <label className="field-label" htmlFor="lesson-selector">选择课程</label>
    <select id="lesson-selector" value={activeId} onChange={(event) => {
      const selected = lessonCatalog.find((lesson) => lesson.id === event.target.value);
      if (selected?.status === 'available') window.location.hash = `lesson/${selected.id}`;
    }} aria-describedby="lesson-availability">
      {lessonCatalog.map((lesson) => <option key={lesson.id} value={lesson.id} disabled={lesson.status !== 'available'}>
        {lesson.category} · {lesson.label}{lesson.status === 'planned' ? '（规划中）' : ''}
      </option>)}
    </select>
    <span id="lesson-availability" className="availability-note">已开放 {availableLessons.length} 门课程；其他结构会按路线图逐步加入。</span>
  </div>;
}
