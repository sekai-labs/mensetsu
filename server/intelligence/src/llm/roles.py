from enum import Enum


class ModelRole(str, Enum):
    EXTRACTION = "extraction"
    RESEARCH = "research"
    SYNTHESIS = "synthesis"
    MATCHING = "matching"
    CURRICULUM = "curriculum"
    CONTENT = "content"
    QUESTIONS = "questions"
    REFINEMENT = "refinement"
    VALIDATION = "validation"
